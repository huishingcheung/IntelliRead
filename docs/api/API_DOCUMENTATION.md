# API 文档

| 项目 | 内容 |
|---|---|
| 文档名称 | API 文档 |
| 项目名称 | IntelliRead |
| 负责人 | 成员 B |
| 状态 | Implemented |
| 最后更新 | 2026-06-29 |

Base URL：`/api/v1`。受保护接口使用 `Authorization: Bearer <JWT>`。

成功响应：`{"success":true,"data":...}`。错误响应：`{"success":false,"error":{"code":"...","message":"..."}}`。

| 方法 | 路径 | 鉴权 | 请求 | 说明 |
|---|---|---|---|---|
| `GET` | `/health` | 否 | 无 | 健康检查 |
| `POST` | `/auth/register` | 否 | JSON：`username,email,password` | 注册用户，成功返回 `201` |
| `POST` | `/auth/login` | 否 | JSON：`email,password` | 返回 JWT 与用户信息 |
| `POST` | `/documents` | 是 | multipart：`title?`,`file` | 导入 Markdown/TXT 并解析段落，成功返回 `201` |
| `GET` | `/documents?q=&limit=&offset=&archived=&tag_id=` | 是 | Query | 当前用户文档列表；搜索标题/正文；归档和标签筛选 |
| `GET` | `/documents/{id}` | 是 | Path | 当前用户文档详情和段落 |
| `PATCH` | `/documents/{id}` | 是 | JSON：`title?`,`archived?` | 更新标题或归档状态 |
| `DELETE` | `/documents/{id}` | 是 | Path | 删除文档及关联数据 |
| `PUT` | `/documents/{id}/progress` | 是 | JSON：`paragraph_position,progress_percent` | 新建或覆盖阅读进度 |
| `GET` | `/documents/{id}/progress` | 是 | Path | 读取当前用户阅读进度；尚未记录时 `data` 为 `null` |
| `POST/GET` | `/tags` | 是 | JSON：`name` / 无 | 创建或列出当前用户标签 |
| `PUT/DELETE` | `/tags/{id}` | 是 | JSON：`name` / 无 | 重命名或删除标签 |
| `PUT/GET` | `/documents/{id}/tags` | 是 | JSON：`tag_ids` / 无 | 覆盖或读取文档标签 |
| `POST/GET` | `/documents/{id}/notes` | 是 | JSON：`paragraph_id?`,`content` / 无 | 创建或列出笔记 |
| `PUT/DELETE` | `/notes/{id}` | 是 | JSON：`content` / 无 | 更新或删除笔记 |
| `POST/GET` | `/documents/{id}/highlights` | 是 | JSON：段落、字符范围、颜色 / 无 | 创建或列出高亮 |
| `PUT/DELETE` | `/highlights/{id}` | 是 | JSON：范围/颜色 / 无 | 更新或删除高亮 |
| `GET` | `/statistics/overview` | 是 | 无 | 文档、段落、标签、标注和平均进度概览 |
| `POST` | `/ai/selection` | 是 | JSON：`text,paragraph?,document_title?,source_language?,target_language?` | 分析划词文本，返回辅助翻译、术语、句子结构和 prompt 元数据 |
| `POST` | `/ai/document` | 是 | JSON：`document_id?,title,paragraphs,target_language?` | 对前端传入的文档段落做无状态分析，返回摘要、高频词、术语和阅读建议 |
| `GET` | `/vocabulary?page=&page_size=&sort=&order=&mastery_status=&document_id=` | 是 | Query | 当前用户生词卡列表，支持分页、过滤和排序 |
| `POST` | `/vocabulary` | 是 | JSON：`document_id,paragraph_id?,term,pronunciation?,definition,example_sentence?,source_text?` | 创建生词卡；重复词汇返回 `409` |
| `GET` | `/vocabulary/{id}` | 是 | Path | 读取当前用户自己的生词卡 |
| `PATCH` | `/vocabulary/{id}` | 是 | JSON：`definition?,example_sentence?,mastery_status?` | 更新生词卡 |
| `DELETE` | `/vocabulary/{id}` | 是 | Path | 删除生词卡 |
| `GET` | `/review/queue?limit=&document_id=` | 是 | Query | 获取当前用户待复习词汇队列 |
| `POST` | `/review/answer` | 是 | JSON：`vocabulary_id,answer_result` | 提交复习答题结果，返回 `mastery_status` 和 `next_review_at` |

运行服务后，OpenAPI JSON 位于 `/api-docs/openapi.json`，Swagger UI 位于 `/swagger-ui`。AI 接口示例见 [AI_API_USAGE.md](AI_API_USAGE.md)。

## 导入约束

- 扩展名：`.md`、`.markdown`、`.txt`。
- 编码：UTF-8。
- 大小：由 `MAX_DOCUMENT_BYTES` 控制，默认 `2097152`。
- 段落：以一个或多个空行分隔，空段落丢弃，位置从 `0` 开始。

高亮偏移量按 Unicode 字符计数，允许颜色为 `yellow`、`green`、`blue`、`pink`、`purple`。UTF-8 BOM 会在导入时移除。

JSON、Query、multipart、未知路由和不支持的方法均使用统一 JSON 错误结构。
## 词汇/复习响应示例

所有接口仍沿用统一响应结构。

`GET /api/v1/vocabulary` 返回：

```json
{"success":true,"data":{"items":[],"page":1,"page_size":20,"total":0}}
```

`POST /api/v1/review/answer` 返回：

```json
{"success":true,"data":{"id":"answer-id","vocabulary_id":"vocab-id","answer_result":"good","mastery_status":"familiar","reviewed_at":"2026-06-27T12:00:00Z","next_review_at":"2026-06-30T12:00:00Z"}}
```
