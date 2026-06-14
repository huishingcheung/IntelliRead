# 安全设计

| 项目 | 内容 |
|---|---|
| 文档名称 | 安全设计 |
| 项目名称 | IntelliRead |
| 负责人 | 成员 B |
| 状态 | Implemented |
| 最后更新 | 2026-06-12 |

| 风险 | 控制措施 |
|---|---|
| 密码泄露 | Argon2 随机盐哈希，仅存储编码后的哈希 |
| Token 伪造 | HMAC JWT；启动时要求至少 32 字符密钥；校验过期时间 |
| 越权读取 | 文档查询始终同时匹配 `id` 与 JWT 中的 `user_id` |
| SQL 注入 | SQLx 参数绑定，不拼接用户值 |
| 恶意上传 | 仅 `.md`、`.markdown`、`.txt`；要求 UTF-8；默认最大 2 MiB |
| 跨站 API 调用 | `CORS_ALLOWED_ORIGINS` 显式白名单；仅允许必要方法和请求头 |
| 敏感信息提交 | `.env`、数据库文件和 Token 文件由 `.gitignore` 排除 |
| 内部信息泄露 | 数据库和内部错误仅写日志，对外统一返回 `INTERNAL_ERROR` |

本地默认 Origin 为 `http://localhost:5173`。生产部署必须通过逗号分隔的 `CORS_ALLOWED_ORIGINS` 提供真实 HTTPS 前端域名，不允许使用通配符。
