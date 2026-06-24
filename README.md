# IntelliRead

IntelliRead 是一个基于 Rust 与 AI 驱动的外语文献阅读与词汇学习平台。当前仓库整合 Rust/Axum 后端和 React/Vite 前端，核心目标是让用户在阅读英文论文、技术文档等长文本时减少查词打断，并把阅读过程中的重点内容沉淀为可复习的学习资产。

## 当前功能

- 用户注册、登录、退出登录
- 文献导入、列表、搜索、归档和删除
- Markdown/TXT 文献阅读；可提取文本的 PDF 会先在浏览器端转换为文本后上传
- 沉浸式阅读页、段落导航和阅读进度记录
- 文献标签、阅读笔记、划词高亮和高亮记录
- 学习概览统计

AI 辅助面板目前保留前端交互位置，真实 AI 解析、词汇提取和复习系统接口仍待后续契约确认。

## 项目结构

```text
IntelliRead/
├─ apps/
│  └─ web/                 # React + Vite 前端
├─ backend/                # Rust + Axum 后端
├─ docs/                   # 项目文档
├─ Cargo.toml              # Rust workspace
└─ README.md
```

## 本地启动

### 后端

```powershell
Copy-Item .\backend\.env.example .\backend\.env
# 将 backend/.env 中的 JWT_SECRET 替换为至少 32 位随机值
cargo run -p intelliread-backend
```

后端默认地址：

- `http://127.0.0.1:3000`
- 健康检查：`GET /api/v1/health`
- OpenAPI：`/api-docs/openapi.json`
- Swagger UI：`/swagger-ui`

### 前端

```powershell
cd .\apps\web
npm install
npm run dev
```

前端默认地址：`http://localhost:5173`。

如需修改 API 地址，可复制或编辑 `apps/web/.env.example` 中的 `VITE_API_BASE_URL`。

## 质量检查

GitHub Actions 会在 Pull Request 中执行：

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo build --all-features`
- `npm ci`
- `npm run lint`
- `npm run build`

## 提交注意

不要提交本地环境、数据库、依赖目录或构建产物，包括：

- `backend/.env`
- `data/`、`backend/data/`、`*.db`
- `target/`、`target-review/`
- `apps/web/node_modules/`
- `apps/web/dist/`

## 文档

- [后端架构](docs/design/BACKEND_ARCHITECTURE.md)
- [后端启动说明](backend/README.md)
- [数据库设计](docs/design/DATABASE_DESIGN.md)
- [安全设计](docs/design/SECURITY_DESIGN.md)
- [API 文档](docs/api/API_DOCUMENTATION.md)
- [测试计划](docs/testing/TEST_PLAN.md)
- [部署说明](docs/deployment/DEPLOYMENT.md)
