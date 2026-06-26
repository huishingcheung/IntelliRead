# Vocabulary and Review API Contract

## 状态

草案。本文档用于约定词汇卡片与复习模块的第一版 API 契约。后续 migration、后端接口、前端页面和测试都应基于该契约实现。

## 目标

词汇/复习模块用于将 AI 阅读助手提取出的核心生词、短语和专业术语保存为用户自己的生词卡。用户可以收藏词汇、查看释义和例句、进行复习答题，并根据答题结果自动计算下一次复习时间。

## 数据归属与用户隔离

所有私有资源必须通过 `user_id` 进行隔离。

用户只能访问：

- 属于自己的生词卡
- 属于自己的复习队列
- 属于自己的答题记录

后端不能返回其他用户的词汇或复习数据。

## 生词卡字段

| 字段 | 类型 | 是否必填 | 说明 |
|---|---|---|---|
| `id` | string | 是 | 生词卡 ID |
| `user_id` | string | 是 | 所属用户 ID |
| `document_id` | string | 是 | 来源文献 ID |
| `paragraph_id` | string | 否 | 来源段落 ID |
| `term` | string | 是 | 单词、短语或专业术语 |
| `pronunciation` | string | 否 | 音标或发音信息 |
| `definition` | string | 是 | 释义或解释 |
| `example_sentence` | string | 否 | 原文或 AI 生成的例句 |
| `source_text` | string | 否 | 术语所在的原文上下文 |
| `mastery_status` | string | 是 | 掌握状态：`new`、`learning`、`familiar`、`mastered` |
| `next_review_at` | string | 否 | 下次复习时间 |
| `created_at` | string | 是 | 创建时间 |
| `updated_at` | string | 是 | 更新时间 |

## 复习答题记录字段

| 字段 | 类型 | 是否必填 | 说明 |
|---|---|---|---|
| `id` | string | 是 | 答题记录 ID |
| `user_id` | string | 是 | 所属用户 ID |
| `vocabulary_id` | string | 是 | 对应的生词卡 ID |
| `answer_result` | string | 是 | 答题结果：`wrong`、`hard`、`good`、`easy` |
| `reviewed_at` | string | 是 | 本次复习时间 |
| `next_review_at` | string | 是 | 根据答题结果计算出的下次复习时间 |

## 接口设计

### 获取生词卡列表

`GET /api/vocabulary`

查询参数：

| 参数名 | 类型 | 是否必填 | 说明 |
|---|---|---|---|
| `page` | number | 否 | 页码，默认 `1` |
| `page_size` | number | 否 | 每页数量，默认 `20` |
| `sort` | string | 否 | 排序字段，例如 `created_at` 或 `next_review_at` |
| `order` | string | 否 | 排序方向：`asc` 或 `desc` |
| `mastery_status` | string | 否 | 按掌握状态筛选 |
| `document_id` | string | 否 | 按来源文献筛选 |

响应示例：

```json
{
  "items": [],
  "page": 1,
  "page_size": 20,
  "total": 0
}
```

### 创建生词卡

`POST /api/vocabulary`

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

### 获取单个生词卡

`GET /api/vocabulary/{id}`

后端必须校验该生词卡是否属于当前登录用户。

### 更新生词卡

`PATCH /api/vocabulary/{id}`

请求示例：

```json
{
  "definition": "更新后的释义。",
  "example_sentence": "Updated example.",
  "mastery_status": "learning"
}
```

### 删除生词卡

`DELETE /api/vocabulary/{id}`

只允许删除当前用户自己的生词卡。

### 获取复习队列

`GET /api/review/queue`

查询参数：

| 参数名 | 类型 | 是否必填 | 说明 |
|---|---|---|---|
| `limit` | number | 否 | 返回数量，默认 `20` |
| `document_id` | string | 否 | 按来源文献筛选 |

返回规则：

- 返回 `next_review_at` 为空，或早于当前时间的词汇。
- 默认不返回 `mastery_status = mastered` 的词汇。
- 结果必须只包含当前用户的数据。

### 提交复习答题结果

`POST /api/review/answer`

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
  "vocabulary_id": "vocab_001",
  "answer_result": "good",
  "mastery_status": "familiar",
  "next_review_at": "2026-06-27T12:00:00Z"
}
```

## 第一版复习时间规则

| 答题结果 | 下次复习间隔 | 建议掌握状态 |
|---|---|---|
| `wrong` | 10 分钟后 | `learning` |
| `hard` | 1 天后 | `learning` |
| `good` | 3 天后 | `familiar` |
| `easy` | 7 天后 | `mastered` |

## 错误处理

接口应沿用项目已有错误响应格式，并参考 `docs/api/ERROR_CODES.md` 中定义的错误码。

常见错误场景：

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
- 避免同一用户在同一文献下重复添加相同术语。
- 已加入的术语需要显示状态。
- 加入后可以跳转或引导进入词汇复习页面。

## 测试要求

至少补充以下测试：

- 成功创建生词卡。
- 缺少必填字段时返回错误。
- 生词列表只返回当前用户的数据。
- 不能访问其他用户的生词卡。
- 能正确生成待复习队列。
- 提交答题结果后能更新 `next_review_at`。
- 分页和排序逻辑正确。