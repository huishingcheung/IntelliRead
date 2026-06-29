# 词汇与复习 API 契约

## 状态

已实现。本文件记录词汇卡片与复习模块的第一版 API 契约。当前 migration、后端接口、前端页面和测试均已按该契约落地。

## 目标

词汇/复习模块用于把 AI 阅读助手提取出的核心生词、短语和专业术语保存为用户自己的生词卡。用户可以收藏词汇、查看释义和例句、进行复习答题，并根据答题结果自动计算下一次复习时间。

## 通用约定

- Base URL：`/api/v1`
- 受保护接口必须携带 `Authorization: Bearer <JWT>`。
- 成功响应统一使用 `{"success":true,"data":...}`。
- 错误响应统一使用 `{"success":false,"error":{"code":"...","message":"..."}}`。
- 所有私有资源必须通过 `user_id` 进行用户隔离。

## 数据归属与用户隔离

用户只能访问：

- 属于自己的生词卡
- 属于自己的复习队列
- 属于自己的答题记录

后端不能返回、更新或删除其他用户的词汇或复习数据。查询、更新、删除和复习答题都必须带上当前登录用户的 `user_id` 条件。

## 生词卡字段

| 字段 | 类型 | 是否必填 | 说明 |
|---|---|---|---|
| `id` | string | 是 | 生词卡 ID |
| `user_id` | string | 是 | 所属用户 ID，仅后端存储，不直接由客户端提交 |
| `document_id` | string | 是 | 来源文献 ID |
| `paragraph_id` | string | 否 | 来源段落 ID |
| `term` | string | 是 | 单词、短语或专业术语 |
| `pronunciation` | string | 否 | 音标或发音信息 |
| `definition` | string | 是 | 释义或解释 |
| `example_sentence` | string | 否 | 原文或 AI 生成的例句 |
| `source_text` | string | 否 | 术语所在的原文上下文 |
| `mastery_status` | string | 是 | 掌握状态：`new`、`learning`、`familiar`、`mastered` |
| `next_review_at` | string | 否 | 下次复习时间，RFC 3339 字符串 |
| `created_at` | string | 是 | 创建时间 |
| `updated_at` | string | 是 | 更新时间 |

## 复习答题记录字段

| 字段 | 类型 | 是否必填 | 说明 |
|---|---|---|---|
| `id` | string | 是 | 答题记录 ID |
| `user_id` | string | 是 | 所属用户 ID，仅后端存储，不直接由客户端提交 |
| `vocabulary_id` | string | 是 | 对应的生词卡 ID |
| `answer_result` | string | 是 | 答题结果：`wrong`、`hard`、`good`、`easy` |
| `mastery_status` | string | 是 | 根据答题结果得到的新掌握状态 |
| `reviewed_at` | string | 是 | 本次复习时间 |
| `next_review_at` | string | 是 | 根据答题结果计算出的下次复习时间 |

## 接口设计

### 获取生词卡列表

`GET /api/v1/vocabulary`

查询参数：

| 参数名 | 类型 | 是否必填 | 说明 |
|---|---|---|---|
| `page` | number | 否 | 页码，默认 `1` |
| `page_size` | number | 否 | 每页数量，默认 `20` |
| `sort` | string | 否 | 排序字段，例如 `created_at`、`next_review_at`、`term` |
| `order` | string | 否 | 排序方向：`asc` 或 `desc` |
| `mastery_status` | string | 否 | 按掌握状态筛选 |
| `document_id` | string | 否 | 按来源文献筛选 |

响应示例：

```json
{
  "success": true,
  "data": {
    "items": [],
    "page": 1,
    "page_size": 20,
    "total": 0
  }
}
```

### 创建生词卡

`POST /api/v1/vocabulary`

请求示例：

```json
{
  "document_id": "doc_001",
  "paragraph_id": "para_001",
  "term": "distributed system",
  "pronunciation": "",
  "definition": "由多个网络节点协同工作的系统。",
  "example_sentence": "A distributed system coordinates multiple nodes.",
  "source_text": "Original paragraph text here."
}
```

响应示例：

```json
{
  "success": true,
  "data": {
    "id": "vocab_001",
    "document_id": "doc_001",
    "paragraph_id": "para_001",
    "term": "distributed system",
    "pronunciation": null,
    "definition": "由多个网络节点协同工作的系统。",
    "example_sentence": "A distributed system coordinates multiple nodes.",
    "source_text": "Original paragraph text here.",
    "mastery_status": "new",
    "next_review_at": null,
    "created_at": "2026-06-27T12:00:00Z",
    "updated_at": "2026-06-27T12:00:00Z"
  }
}
```

### 获取单个生词卡

`GET /api/v1/vocabulary/{id}`

后端必须校验该生词卡是否属于当前登录用户。

### 更新生词卡

`PATCH /api/v1/vocabulary/{id}`

请求示例：

```json
{
  "definition": "更新后的释义。",
  "example_sentence": "Updated example.",
  "mastery_status": "learning"
}
```

响应示例：

```json
{
  "success": true,
  "data": {
    "id": "vocab_001",
    "document_id": "doc_001",
    "paragraph_id": "para_001",
    "term": "distributed system",
    "pronunciation": null,
    "definition": "更新后的释义。",
    "example_sentence": "Updated example.",
    "source_text": "Original paragraph text here.",
    "mastery_status": "learning",
    "next_review_at": null,
    "created_at": "2026-06-27T12:00:00Z",
    "updated_at": "2026-06-27T12:05:00Z"
  }
}
```

### 删除生词卡

`DELETE /api/v1/vocabulary/{id}`

只允许删除当前用户自己的生词卡。成功时返回 `204 No Content`。

### 获取复习队列

`GET /api/v1/review/queue`

查询参数：

| 参数名 | 类型 | 是否必填 | 说明 |
|---|---|---|---|
| `limit` | number | 否 | 返回数量，默认 `20` |
| `document_id` | string | 否 | 按来源文献筛选 |

返回规则：

- 返回 `next_review_at` 为空，或早于等于当前时间的词汇。
- 默认不返回 `mastery_status = mastered` 的词汇。
- 结果必须只包含当前用户的数据。

响应示例：

```json
{
  "success": true,
  "data": []
}
```

### 提交复习答题结果

`POST /api/v1/review/answer`

请求示例：

```json
{
  "vocabulary_id": "vocab_001",
  "answer_result": "good"
}
```

响应示例：

```json
{
  "success": true,
  "data": {
    "id": "answer_001",
    "vocabulary_id": "vocab_001",
    "answer_result": "good",
    "mastery_status": "familiar",
    "reviewed_at": "2026-06-27T12:00:00Z",
    "next_review_at": "2026-06-30T12:00:00Z"
  }
}
```

## 第一版复习时间规则

| 答题结果 | 下次复习间隔 | 建议掌握状态 |
|---|---|---|
| `wrong` | 10 分钟后 | `learning` |
| `hard` | 1 天后 | `learning` |
| `good` | 3 天后 | `familiar` |
| `easy` | 7 天后 | `mastered` |

## 第一版复习调度算法

复习队列只返回当前用户自己的生词卡，筛选规则如下：

1. 根据当前登录用户的 `user_id` 查询生词卡。
2. 只返回 `next_review_at` 为空，或 `next_review_at` 小于等于当前时间的词汇。
3. 默认排除 `mastery_status = mastered` 的词汇。
4. 如果传入 `document_id`，则只返回该文献下的词汇。
5. 按 `next_review_at` 升序排列，较早需要复习的词汇排在前面。
6. 根据 `limit` 参数限制返回数量，默认返回 20 条。

伪代码：

```text
review_queue = vocabulary_cards
  where user_id = current_user_id
  and (next_review_at is null or next_review_at <= now)
  and mastery_status != "mastered"
  and document_id matches query if provided
  order by next_review_at asc
  limit query.limit or 20
```

用户提交复习结果后，系统根据 `answer_result` 更新复习状态。

```text
if answer_result == "wrong":
    next_review_at = now + 10 minutes
    mastery_status = "learning"

if answer_result == "hard":
    next_review_at = now + 1 day
    mastery_status = "learning"

if answer_result == "good":
    next_review_at = now + 3 days
    mastery_status = "familiar"

if answer_result == "easy":
    next_review_at = now + 7 days
    mastery_status = "mastered"
```

## 重复生词处理算法

为避免用户重复添加同一个生词，第一版按以下规则判断重复：

1. 同一 `user_id`
2. 同一 `document_id`
3. 相同的 `term`

如果以上三个字段都相同，则认为是重复生词，接口返回 `409 Conflict`。

伪代码：

```text
exists = vocabulary_cards
  where user_id = current_user_id
  and document_id = request.document_id
  and lower(term) = lower(request.term)

if exists:
    return 409 Conflict
```

## 错误处理

接口应沿用项目已有错误响应格式，并参考 `docs/api/ERROR_CODES.md` 中定义的错误码。

| 场景 | HTTP 状态码 |
|---|---|
| 缺少必填字段 | 400 |
| 枚举值非法 | 400 |
| 未登录或认证失败 | 401 |
| 访问其他用户资源 | 404 或 403 |
| 资源不存在 | 404 |
| 同一用户在同一文献下重复添加同一词汇 | 409 |

## 前端接入要求

阅读页面需要支持将 AI 阅读助手提取出的术语加入生词卡。

需要实现：

- 展示 AI 提取出的核心词汇和专业术语。
- 每个术语提供“加入生词卡”操作。
- 避免同一用户在同一文献下重复加入相同术语。
- 已加入的术语需要显示状态。
- 加入后可以跳转或引导进入词汇复习页面。

## 测试要求

至少补充以下测试：

- 成功创建生词卡。
- 缺少必填字段时返回错误。
- 生词列表只返回当前用户的数据。
- 不能访问其他用户的生词卡。
- 能正确生成待复习队列。
- 提交答题结果后能更新 `mastery_status` 和 `next_review_at`。
- 分页和排序逻辑正确。
