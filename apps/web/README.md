# IntelliRead Web

React + TypeScript + Vite 前端，负责 IntelliRead 的登录注册、文献导入、文献库、沉浸式阅读、阅读进度、标签、笔记、高亮、学习概览、生词管理和复习界面。

## Scripts

```powershell
npm install
npm run dev
npm run lint
npm run build
npm run test:e2e
```

默认 API 地址为 `http://127.0.0.1:3000/api/v1`，可通过 `VITE_API_BASE_URL` 覆盖。

PDF 文件在浏览器端提取文本后以 TXT 内容上传；后端接口仍只接收 UTF-8 Markdown/TXT。

AI 分析结果中的术语可以直接加入生词本；`/vocabulary` 提供词汇管理，`/review` 提供到期词汇复习和四级反馈。

Playwright 端到端验收会启动临时后端和前端，使用 `local-deterministic` AI 完成注册、文献导入、术语收藏和复习闭环。该测试默认由 GitHub Actions 执行。
