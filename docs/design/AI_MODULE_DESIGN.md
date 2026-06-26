# AI Module Design

## Module Boundary

The AI module lives in `backend/src/ai/mod.rs` and exposes authenticated APIs under `/api/v1/ai`. It is intentionally stateless in this phase: requests contain selected text or document paragraphs, and responses are computed on demand.

This keeps the first AI contract easy to test and avoids adding database tables before the response shape is stable.

## Features

- Selected-text translation
- Intelligent reading analysis
- Frequent-word extraction
- Professional terminology recognition
- Long-sentence parsing
- Prompt template metadata
- AI provider wrapper shape through the `provider` and `prompt` response fields

## Provider Design

The default provider is `local-deterministic`. It does not call an external LLM. It produces stable output for tests, local demos, and classroom presentation.

The production provider can be switched to DeepSeek V4 Pro through environment variables:

```text
AI_PROVIDER=deepseek
DEEPSEEK_API_KEY=<secret>
AI_API_BASE_URL=https://api.deepseek.com
AI_MODEL=deepseek-v4-pro
AI_THINKING=disabled
```

When DeepSeek is enabled, the backend calls the OpenAI-compatible Chat Completions endpoint and requests strict JSON output. The model-generated content replaces local translation, selection analysis, document summary, and reading suggestions. Deterministic local logic still supplies terminology, frequent words, and sentence structure so the public API shape remains stable.

The API response already includes:

- `provider`: current provider name
- `prompt.name`: prompt template identifier
- `prompt.template`: rendered prompt context

OpenAI-compatible providers can reuse the same request DTOs and replace the selected provider through environment variables such as:

- `AI_PROVIDER`
- `AI_API_BASE_URL`
- `DEEPSEEK_API_KEY` or `AI_API_KEY`
- `AI_MODEL`

## Core Algorithms

### Selected-Text Translation

The local provider recognizes common academic and AI terms from a built-in bilingual glossary. It returns a Chinese reading aid that preserves the English source and explains key terms.

When no glossary term is found, it returns a generic reading strategy instead of pretending to produce a full machine translation.

### Intelligent Analysis

The module estimates reading difficulty from:

- Character length
- Clause count
- Punctuation density
- Number of recognized terms

The response explains the main reading strategy in Chinese.

### Frequent-Word Extraction

The document analyzer:

1. Lowercases English text.
2. Splits on non-letter characters.
3. Removes common stop words.
4. Ignores words shorter than four characters.
5. Sorts by frequency, then alphabetically.

### Terminology Recognition

Terminology combines two signals:

- A built-in glossary for known academic and AI terms.
- Repeated long words with at least two occurrences.

Each term includes `term`, `category`, `explanation`, and `frequency`.

### Long-Sentence Parsing

The sentence parser inserts clause boundaries around common connectors such as `because`, `although`, `while`, `which`, and `that`, then splits on punctuation. The first clause is treated as the main clause and later clauses are treated as supporting material.

## Frontend Integration

`apps/web/src/pages/DocumentReaderPage.tsx` now calls:

- `POST /api/v1/ai/selection` for selected text
- `POST /api/v1/ai/document` for whole-document analysis

The right-side AI panel renders translation, sentence parsing, terminology, frequent words, suggestions, provider, and prompt metadata.

## Limitations

- The default provider is deterministic and heuristic-based; set `AI_PROVIDER=deepseek` to use DeepSeek V4 Pro.
- AI results are not persisted.
- Selection analysis trusts frontend-provided text and context.
- Cross-paragraph selection is still unsupported by the reader interaction.
