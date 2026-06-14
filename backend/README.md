# IntelliRead Backend

Rust/Axum 后端服务，提供认证、文献管理、标签、笔记、高亮、阅读进度、统计与 OpenAPI。

## 启动

在仓库根目录执行：

```powershell
Copy-Item backend/.env.example backend/.env
# 将 JWT_SECRET 替换为至少 32 位随机值
cargo run -p intelliread-backend
```

默认地址为 `http://127.0.0.1:3000`。健康检查：`GET /api/v1/health`；OpenAPI：`/api-docs/openapi.json`；Swagger UI：`/swagger-ui`。

服务启动时会连接 `DATABASE_URL` 并自动执行 `backend/migrations/`。上传只接受 UTF-8 `.md`、`.markdown`、`.txt`，大小由 `MAX_DOCUMENT_BYTES` 限制。

## 验证

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --all-features
```

不要提交 `backend/.env`、数据库文件或真实密钥。
