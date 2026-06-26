# 错误码

| 项目 | 内容 |
|---|---|
| 文档名称 | 错误码 |
| 项目名称 | IntelliRead |
| 负责人 | 成员 B |
| 状态 | Implemented |
| 最后更新 | 2026-06-12 |

| HTTP | code | 含义 |
|---|---|---|
| `400` | `VALIDATION_ERROR` | 输入格式、范围、编码或段落位置非法 |
| `401` | `UNAUTHORIZED` | 缺少、无效或过期 JWT，或凭据错误 |
| `403` | `FORBIDDEN` | 已认证但无权执行操作；当前闭环未直接使用 |
| `404` | `NOT_FOUND` | 资源不存在或不属于当前用户 |
| `409` | `CONFLICT` | 用户名或邮箱重复 |
| `413` | `PAYLOAD_TOO_LARGE` | 上传超过配置限制 |
| `415` | `UNSUPPORTED_MEDIA_TYPE` | 上传扩展名不受支持 |
| `405` | `METHOD_NOT_ALLOWED` | 路径存在但 HTTP 方法不支持 |
| `502` | `UPSTREAM_ERROR` | AI provider 请求失败、超时、返回非 2xx 或响应无法解析 |
| `500` | `INTERNAL_ERROR` | 配置、数据库或未公开内部错误 |
