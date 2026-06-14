# 后端架构设计

| 项目 | 内容 |
|---|---|
| 文档名称 | 后端架构设计 |
| 项目名称 | IntelliRead |
| 负责人 | 成员 B |
| 状态 | Implemented |
| 最后更新 | 2026-06-12 |

## 架构

采用模块化单体，HTTP、业务校验和数据访问按业务模块组织，避免在 MVP 阶段引入微服务与队列。

```mermaid
flowchart LR
    Client[前端或 API 客户端] --> Router[Axum Router]
    Router --> Auth[auth]
    Router --> Documents[documents]
    Router --> Reading[reading]
    Router --> Tags[tags]
    Router --> Annotations[annotations]
    Router --> Statistics[statistics]
    Auth --> DB[(SQLite)]
    Documents --> DB
    Reading --> DB
    Tags --> DB
    Annotations --> DB
    Statistics --> DB
    Router --> OpenAPI[utoipa OpenAPI]
    Router --> Trace[tracing middleware]
```

## 模块边界

| 模块 | 职责 | 所有者 |
|---|---|---|
| `config` | 环境变量与安全默认值 | 成员 B |
| `database` | 连接池、目录创建、migration | 成员 B |
| `auth` | 注册、登录、Argon2、JWT extractor | 成员 B |
| `documents` | 上传限制、段落解析、文档查询与归属校验 | 成员 B |
| `reading` | 阅读位置与百分比 upsert | 成员 B |
| `tags` | 用户标签与文档标签关联 | 成员 B |
| `annotations` | 文档/段落笔记和文本高亮 | 成员 B |
| `statistics` | 当前用户学习数据聚合 | 成员 B |
| `response` / `error` | 统一成功与错误结构 | 成员 B |

AI、词汇与复习模块由其他成员负责业务方案；后端后续仅按确认的 [API 契约](../project-memory/API_CONTRACT.md) 提供接口。

## 请求流程

```mermaid
sequenceDiagram
    participant C as Client
    participant A as Auth Extractor
    participant H as Handler
    participant D as SQLite
    C->>A: Authorization Bearer JWT
    A->>A: 校验签名和过期时间
    A->>H: AuthUser(user_id)
    H->>D: 参数化查询并包含 user_id
    D-->>H: 当前用户资源
    H-->>C: 统一 JSON 响应
```
