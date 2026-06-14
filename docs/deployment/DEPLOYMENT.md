# 部署与配置

| 项目 | 内容 |
|---|---|
| 文档名称 | 部署与配置 |
| 项目名称 | IntelliRead |
| 负责人 | 成员 B |
| 状态 | Implemented |
| 最后更新 | 2026-06-12 |

```powershell
Copy-Item backend/.env.example backend/.env
cargo build --release -p intelliread-backend
./target/release/intelliread-backend.exe
```

必须替换 `JWT_SECRET`，并确保运行账户可写 `DATABASE_URL` 对应目录。首次启动自动执行 `backend/migrations/` 中尚未应用的 migration。

生产环境应由反向代理终止 HTTPS，将 `CORS_ALLOWED_ORIGINS` 设置为真实前端域名，限制请求体大小，并保护数据库目录不被静态服务暴露。
