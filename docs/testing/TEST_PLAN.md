# 测试计划

| 项目 | 内容 |
|---|---|
| 文档名称 | 测试计划 |
| 项目名称 | IntelliRead |
| 负责人 | 成员 B |
| 状态 | Implemented |
| 最后更新 | 2026-06-12 |

| ID | 场景 | 预期 | 位置 |
|---|---|---|---|
| T01 | 注册成功并登录 | 返回用户与 JWT | `backend/tests/api_flow.rs` |
| T02 | 重复用户名 | `409 CONFLICT` | 同上 |
| T03 | 错误密码 | `401 UNAUTHORIZED` | 同上 |
| T04 | 无 Token | `401 UNAUTHORIZED` | 同上 |
| T05 | 无效 Token | `401 UNAUTHORIZED` | 同上 |
| T06 | Markdown 导入 | 保存文档和两个段落 | 同上 |
| T07 | 非法扩展名 | `415` | 同上 |
| T08 | 超大文件 | `413` | 同上 |
| T09 | 用户资源隔离 | 其他用户查询返回 `404` | 同上 |
| T10 | 阅读进度 | 写入后通过 GET 回读并确认 100% | 同上 |
| T11 | 段落拆分单元测试 | CRLF/LF 均正确拆分 | `backend/src/documents/mod.rs` |
| T12 | 空库 migration | 测试初始化成功 | 每个集成测试 setup |
| T13 | 正文搜索与标签筛选 | 返回匹配当前用户文档 | `backend/tests/api_flow.rs` |
| T14 | 文档归档和删除 | 活跃/归档列表正确，删除后不可读 | 同上 |
| T15 | 标签归属隔离 | 其他用户不能给文档设置标签 | 同上 |
| T16 | 笔记归属隔离 | 其他用户不能更新笔记 | 同上 |
| T17 | 高亮范围 | 合法范围成功，越界返回 `400` | 同上 |
| T18 | 学习概览 | 聚合文档、段落、标签、笔记、高亮 | 同上 |
| T19 | CORS 白名单 | 配置 Origin 返回允许头，其他 Origin 不返回允许头 | 同上 |
| T20 | 过期 JWT | 过期 120 秒的 Token 返回 `401` | 同上 |
| T21 | 统一错误 | 畸形 JSON、非法 Query、未知路由和错误方法返回 JSON | 同上 |
| T22 | migration schema | 空库创建预期表，外键检查无异常 | 同上 |
| T23 | 空文件和非 UTF-8 | 返回 `400 VALIDATION_ERROR` | 同上 |
| V01 | 创建生词卡 | 返回 `200` 和生词卡数据 | `backend/tests/api_flow.rs` |
| V02 | 重复生词卡 | 同一用户、文献、词汇重复时返回 `409` | 同上 |
| V03 | 生词卡必填字段校验 | 缺少必填字段返回 `400` | 同上 |
| V04 | 生词/复习鉴权 | 未登录请求返回 `401` | 同上 |
| V05 | 用户隔离 | 跨用户读取或答题返回 `404` | 同上 |
| V06 | 非法枚举 | 非法 `mastery_status` 或 `answer_result` 返回 `400` | 同上 |
| V07 | PATCH/DELETE 生词卡 | 更新成功；删除后再次读取返回 `404` | 同上 |
| V08 | 分页排序 | 多条生词按分页和排序参数稳定返回 | 同上 |
| V09 | 复习队列与答题 | 队列返回待复习词汇；答题后更新 `mastery_status` 和 `next_review_at` | 同上 |
| E01 | 浏览器学习闭环 | 注册、导入、AI 分析、术语收藏和复习均可在真实 Chromium 中完成 | `apps/web/e2e/learning-flow.spec.ts` |

质量门禁为 `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-features`、`cargo build --all-features`、`npm run lint`、`npm run build` 和 `npm run test:e2e`。Playwright 验收默认只在 GitHub Actions 的临时环境执行。
