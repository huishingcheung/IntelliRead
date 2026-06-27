# 测试报告

| 项目 | 内容 |
|---|---|
| 文档名称 | 测试报告 |
| 项目名称 | IntelliRead |
| 负责人 | 成员 B |
| 状态 | Verified |
| 最后更新 | 2026-06-12 |

## 当前结果

2026-06-12 使用 Rust `1.96.0`、`x86_64-pc-windows-gnu` 实际执行质量门禁。Rustup、Cargo、工具链和 crate 缓存位于 `E:\DevTools\Rust`，构建产物位于仓库的 `target/`（E 盘）。

| 命令 | 结果 |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed，零警告 |
| `cargo test --all-features` | Passed：1 个单元测试、14 个 API 集成测试、0 失败、0 ignored |
| `cargo build --all-features` | Passed |

最终验收回归后为 1 个单元测试、14 个 API 集成测试、0 失败、0 ignored。新增覆盖阅读进度回读、过期 JWT、统一 JSON 错误、空文件、非 UTF-8、migration schema 与外键检查。

首次 Clippy 构建发现 Swagger UI 构建脚本依赖 GitHub 下载，已启用 `vendored` feature 后重新执行并通过。
## 词汇/复习验证

2026-06-27 已补充词汇/复习后端集成测试，并通过 `cargo test --all-features`。覆盖范围包括创建生词卡、重复检测、必填字段校验、未登录请求、用户隔离、跨用户答题、非法枚举、PATCH、DELETE、真实分页、多数据排序、复习队列和复习答题调度。

## 浏览器端到端验收

2026-06-27 已新增 Playwright Chromium 验收流程，覆盖注册、TXT 文献导入、确定性 AI 文献分析、术语加入生词本和复习反馈。[GitHub Actions run #24](https://github.com/huishingcheung/IntelliRead/actions/runs/28288081092) 已验证 Backend、Frontend 和 End-to-end 三个 job 全部通过，其中浏览器学习闭环 1 个用例通过。
