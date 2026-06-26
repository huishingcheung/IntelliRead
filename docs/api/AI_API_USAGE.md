# AI API Usage

All AI endpoints require the same Bearer JWT authentication used by the rest of IntelliRead.

Base URL:

```text
http://127.0.0.1:3000/api/v1
```

## Analyze Selected Text

```http
POST /api/v1/ai/selection
Authorization: Bearer <token>
Content-Type: application/json
```

Request:

```json
{
  "text": "The algorithm improves neural network performance because the dataset is noisy.",
  "paragraph": "The algorithm improves neural network performance because the dataset is noisy.",
  "document_title": "Machine Learning Notes",
  "source_language": "en",
  "target_language": "zh-CN"
}
```

Response:

```json
{
  "success": true,
  "data": {
    "original_text": "The algorithm improves neural network performance because the dataset is noisy.",
    "translation": "面向 zh-CN 的辅助翻译...",
    "analysis": "识别到 4 个专业术语...",
    "terms": [
      {
        "term": "algorithm",
        "category": "computer science",
        "explanation": "算法",
        "frequency": 1
      }
    ],
    "sentence_analysis": {
      "difficulty": "medium",
      "main_clause": "The algorithm improves neural network performance",
      "clauses": [],
      "strategy": []
    },
    "prompt": {
      "name": "selection_translate",
      "template": "Analyze selected academic text..."
    },
    "provider": "local-deterministic"
  }
}
```

Empty `text` returns `400 VALIDATION_ERROR`.

## Analyze Whole Document

```http
POST /api/v1/ai/document
Authorization: Bearer <token>
Content-Type: application/json
```

Request:

```json
{
  "document_id": "doc-1",
  "title": "Neural Network Reading",
  "paragraphs": [
    "The neural network algorithm improves model performance.",
    "The dataset improves model evaluation and the algorithm reduces noisy features."
  ],
  "target_language": "zh-CN"
}
```

Response:

```json
{
  "success": true,
  "data": {
    "document_id": "doc-1",
    "title": "Neural Network Reading",
    "summary": "Neural Network Reading 包含 2 个段落...",
    "frequent_words": [
      {
        "word": "algorithm",
        "count": 2
      }
    ],
    "terminology": [
      {
        "term": "neural network",
        "category": "machine learning",
        "explanation": "神经网络",
        "frequency": 1
      }
    ],
    "suggestions": [
      "先快速浏览标题和段首句，建立主题框架。"
    ],
    "prompt": {
      "name": "document_summary",
      "template": "Analyze the document..."
    },
    "provider": "local-deterministic"
  }
}
```

Empty `title` or empty `paragraphs` returns `400 VALIDATION_ERROR`.

## Prompt Templates

Current prompt names:

- `selection_translate`
- `document_summary`

The response includes the rendered prompt metadata so future real-model calls can be audited and debugged.

## Local Provider

`local-deterministic` is the default implementation. It supports demos without an API key and keeps tests stable.

Future real AI configuration should be added through environment variables:

```text
AI_PROVIDER=openai-compatible
AI_API_BASE_URL=https://api.example.com/v1
AI_API_KEY=<secret>
AI_MODEL=<model-name>
```
