# IntelliRead Web

React + TypeScript + Vite 前端，负责 IntelliRead 的登录注册、文献导入、文献库、沉浸式阅读、阅读进度、标签、笔记、高亮和学习概览界面。

## Scripts

```powershell
npm install
npm run dev
npm run lint
npm run build
```

默认 API 地址为 `http://127.0.0.1:3000/api/v1`，可通过 `VITE_API_BASE_URL` 覆盖。

PDF 文件在浏览器端提取文本后以 TXT 内容上传；后端接口仍只接收 UTF-8 Markdown/TXT。
