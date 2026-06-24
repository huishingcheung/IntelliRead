# IntelliRead

[![CI](https://github.com/huishingcheung/IntelliRead/actions/workflows/ci.yml/badge.svg)](https://github.com/huishingcheung/IntelliRead/actions/workflows/ci.yml)

IntelliRead 是一个基于 Rust 与 AI 驱动的外语文献阅读与词汇学习平台。项目面向英文论文、技术文档等长文本阅读场景，目标是在阅读过程中减少频繁查词带来的打断，并把高亮、笔记、标签和后续复习资料沉淀为可持续使用的学习资产。

当前主线已经整合 Rust/Axum 后端与 React/Vite 前端，核心阅读闭环可用于课程演示；AI 解析、词汇提取和复习系统仍处于接口契约确认阶段。

## 当前状态

| 模块 | 状态 | 说明 |
|---|---|---|
| 后端基础架构 | 已完成 | Rust workspace、Axum、SQLite/SQLx migration、OpenAPI、统一 JSON 响应 |
| 认证 | 已完成 | 注册、登录、Argon2 密码哈希、Bearer JWT |
| 文献管理 | 已完成 | Markdown/TXT 导入、列表、详情、搜索、归档、删除 |
| 阅读体验 | 已完成 | 段落解析、阅读进度写入与回读、沉浸式阅读页 |
| 标签与笔记 | 已完成 | 标签 CRUD、文献标签绑定、文献/段落笔记 |
| 高亮 | 已完成 | 段落内字符范围高亮、颜色更新、删除 |
| 学习统计 | 已完成 | 当前用户文献、段落、标签、笔记、高亮和平均进度统计 |
| 前端界面 | 已完成 | 登录注册、首页、文献库、阅读页、标签笔记页 |
| CI | 已完成 | GitHub Actions 覆盖后端格式/检查/测试/构建和前端 lint/build |
| AI/词汇/复习 | 待确认 | 需要先确认跨成员接口契约，再实现 migration 和 API |

## 功能范围

- 用户注册、登录、退出登录
- 文献导入、列表、搜索、归档、删除
- Markdown/TXT 文献阅读
- PDF 文献在浏览器端提取文本后，以 TXT 内容上传到后端
- 沉浸式阅读页、段落导航、阅读进度自动记录
- 文献标签、阅读笔记、划词高亮、高亮记录
- 学习数据概览统计
- Swagger UI 和 OpenAPI JSON

> 注意：后端上传接口只接收 UTF-8 Markdown/TXT。PDF 支持属于前端预处理能力，不代表后端直接解析 PDF。

## 技术栈

### 后端

- Rust stable
- Axum
- Tokio
- SQLite
- SQLx
- Serde
- JWT
- Argon2
- tracing
- utoipa / Swagger UI

### 前端

- React
- TypeScript
- Vite
- Tailwind CSS
- React Router
- pdfjs-dist
- GSAP

## 项目结构

```text
IntelliRead/
├─ apps/
│  └─ web/                 # React + Vite 前端
├─ backend/                # Rust + Axum 后端
│  ├─ migrations/          # SQLx migration
│  ├─ src/                 # 后端源码
│  └─ tests/               # 后端集成测试
├─ docs/                   # API、架构、数据库、安全、测试和部署文档
├─ Cargo.toml              # Rust workspace
├─ Cargo.lock
└─ README.md
```

## 本地启动

### 1. 后端

前置条件：Rust stable、Cargo。

```powershell
Copy-Item .\backend\.env.example .\backend\.env
# 将 backend/.env 中的 JWT_SECRET 替换为至少 32 位随机值
cargo run -p intelliread-backend
```

后端默认地址：

- API Base URL: `http://127.0.0.1:3000/api/v1`
- Health Check: `GET /api/v1/health`
- OpenAPI: `http://127.0.0.1:3000/api-docs/openapi.json`
- Swagger UI: `http://127.0.0.1:3000/swagger-ui`

服务启动时会连接 `DATABASE_URL` 并自动执行 `backend/migrations/` 中尚未应用的 migration。

### 2. 前端

前置条件：Node.js、npm。

```powershell
cd .\apps\web
npm install
npm run dev
```

前端默认地址：`http://localhost:5173`。

如需修改 API 地址，可在 `apps/web/.env.example` 的基础上配置：

```text
VITE_API_BASE_URL=http://127.0.0.1:3000/api/v1
```

## 环境变量

后端环境变量见 `backend/.env.example`。

| 变量 | 说明 |
|---|---|
| `DATABASE_URL` | SQLite 数据库连接地址 |
| `JWT_SECRET` | JWT 签名密钥，必须至少 32 字符 |
| `JWT_EXPIRATION_SECONDS` | Token 有效期 |
| `SERVER_HOST` | 后端监听地址 |
| `SERVER_PORT` | 后端监听端口 |
| `MAX_DOCUMENT_BYTES` | 单个文献上传大小限制 |
| `CORS_ALLOWED_ORIGINS` | 允许访问后端的前端 Origin，逗号分隔 |

## 质量检查

GitHub Actions 会在 Pull Request 和 `main` 推送时执行以下检查：

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --all-features
```

```powershell
cd .\apps\web
npm ci
npm run lint
npm run build
```

## 当前协作重点

下一阶段应先确认 AI、词汇和复习模块的接口契约，再进入后端实现。建议至少明确：

- AI：划词翻译、长难句解析、文档分析的请求/响应字段
- 词汇：生词卡字段、来源文献/段落、释义、例句、掌握状态
- 复习：复习队列、答题结果、下次复习时间、状态流转
- 通用规则：鉴权、错误码、用户隔离、分页和排序

在契约确认前，不建议直接新增数据库表或写死前端字段。

## 提交注意

不要提交本地环境、数据库、依赖目录或构建产物，包括：

- `backend/.env`
- `data/`
- `backend/data/`
- `*.db`
- `*.db-shm`
- `*.db-wal`
- `target/`
- `target-review/`
- `apps/web/node_modules/`
- `apps/web/dist/`

## 文档导航

- [后端启动说明](backend/README.md)
- [前端说明](apps/web/README.md)
- [API 文档](docs/api/API_DOCUMENTATION.md)
- [错误码](docs/api/ERROR_CODES.md)
- [后端架构](docs/design/BACKEND_ARCHITECTURE.md)
- [数据库设计](docs/design/DATABASE_DESIGN.md)
- [安全设计](docs/design/SECURITY_DESIGN.md)
- [测试计划](docs/testing/TEST_PLAN.md)
- [测试报告](docs/testing/TEST_REPORT.md)
- [部署说明](docs/deployment/DEPLOYMENT.md)
