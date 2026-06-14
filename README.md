# IntelliRead

IntelliRead 是基于 Rust 与 AI 驱动的外语文献阅读与词汇学习平台。本仓库当前由成员 B 负责后端核心，已实现注册登录、JWT 鉴权、文档导入与管理、正文搜索、归档、标签、笔记、高亮、阅读进度写入与回读以及学习概览统计。AI、词汇与复习业务契约仍待成员 C/E 确认。

## 快速开始

前置条件：Rust stable、Cargo。

```powershell
Copy-Item backend/.env.example backend/.env
# 将 backend/.env 中的 JWT_SECRET 替换为至少 32 位的随机值
cargo run -p intelliread-backend
```

服务默认监听 `http://127.0.0.1:3000`，健康检查为 `GET /api/v1/health`，Swagger UI 为 `/swagger-ui`。

## 质量检查

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --all-features
```

本项目已使用 Rust `1.96.0` GNU 工具链完成格式、Clippy 和测试验证。真实结果见 [测试报告](docs/testing/TEST_REPORT.md)。

## 文档

- [后端架构](docs/design/BACKEND_ARCHITECTURE.md)
- [后端启动说明](backend/README.md)
- [数据库设计](docs/design/DATABASE_DESIGN.md)
- [安全设计](docs/design/SECURITY_DESIGN.md)
- [API 文档](docs/api/API_DOCUMENTATION.md)
- [测试计划](docs/testing/TEST_PLAN.md)
- [部署说明](docs/deployment/DEPLOYMENT.md)
