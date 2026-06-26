use std::collections::{HashMap, HashSet};

use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    auth::AuthUser,
    error::AppError,
    response::{ApiJson, ApiResponse},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct SelectionAnalysisRequest {
    pub text: String,
    pub paragraph: Option<String>,
    pub document_title: Option<String>,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DocumentAnalysisRequest {
    pub document_id: Option<String>,
    pub title: String,
    pub paragraphs: Vec<String>,
    pub target_language: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PromptInfo {
    pub name: &'static str,
    pub template: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TermInfo {
    pub term: String,
    pub category: String,
    pub explanation: String,
    pub frequency: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClauseAnalysis {
    pub role: String,
    pub text: String,
    pub note: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SentenceAnalysis {
    pub difficulty: String,
    pub main_clause: String,
    pub clauses: Vec<ClauseAnalysis>,
    pub strategy: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SelectionAnalysisResponse {
    pub original_text: String,
    pub translation: String,
    pub analysis: String,
    pub terms: Vec<TermInfo>,
    pub sentence_analysis: SentenceAnalysis,
    pub prompt: PromptInfo,
    pub provider: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FrequentWord {
    pub word: String,
    pub count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentAnalysisResponse {
    pub document_id: Option<String>,
    pub title: String,
    pub summary: String,
    pub frequent_words: Vec<FrequentWord>,
    pub terminology: Vec<TermInfo>,
    pub suggestions: Vec<String>,
    pub prompt: PromptInfo,
    pub provider: &'static str,
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/selection",
    request_body = SelectionAnalysisRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = ApiResponse<SelectionAnalysisResponse>),
        (status = 400, body = crate::response::ErrorBody),
        (status = 401, body = crate::response::ErrorBody)
    )
)]
pub async fn analyze_selection(
    _user: AuthUser,
    ApiJson(request): ApiJson<SelectionAnalysisRequest>,
) -> Result<(StatusCode, Json<ApiResponse<SelectionAnalysisResponse>>), AppError> {
    let text = request.text.trim();
    if text.is_empty() {
        return Err(AppError::Validation("selected text cannot be empty".into()));
    }

    let target_language = request.target_language.as_deref().unwrap_or("zh-CN");
    let terms = detect_terms(text);
    let sentence_analysis = analyze_sentence(text, terms.len());
    let translation = translate_selection(text, &terms, target_language);
    let analysis = build_selection_analysis(&terms, &sentence_analysis);
    let prompt = prompt_info(
        "selection_translate",
        &[
            ("text", text),
            (
                "document_title",
                request.document_title.as_deref().unwrap_or("Untitled"),
            ),
            ("target_language", target_language),
            (
                "source_language",
                request.source_language.as_deref().unwrap_or("auto"),
            ),
        ],
    );

    Ok((
        StatusCode::OK,
        Json(ApiResponse::new(SelectionAnalysisResponse {
            original_text: text.to_string(),
            translation,
            analysis,
            terms,
            sentence_analysis,
            prompt,
            provider: "local-deterministic",
        })),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/document",
    request_body = DocumentAnalysisRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = ApiResponse<DocumentAnalysisResponse>),
        (status = 400, body = crate::response::ErrorBody),
        (status = 401, body = crate::response::ErrorBody)
    )
)]
pub async fn analyze_document(
    _user: AuthUser,
    ApiJson(request): ApiJson<DocumentAnalysisRequest>,
) -> Result<(StatusCode, Json<ApiResponse<DocumentAnalysisResponse>>), AppError> {
    let document_id = request.document_id.clone();
    let title = request.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation(
            "document title cannot be empty".into(),
        ));
    }

    let paragraphs: Vec<String> = request
        .paragraphs
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if paragraphs.is_empty() {
        return Err(AppError::Validation(
            "document paragraphs cannot be empty".into(),
        ));
    }

    let joined = paragraphs.join("\n");
    let frequent_words = frequent_words(&joined, 8);
    let terminology = detect_terms(&joined);
    let summary = build_document_summary(title, paragraphs.len(), &frequent_words, &terminology);
    let suggestions = build_document_suggestions(&frequent_words, &terminology);
    let target_language = request.target_language.as_deref().unwrap_or("zh-CN");
    let prompt = prompt_info(
        "document_summary",
        &[
            ("title", title),
            ("target_language", target_language),
            (
                "analysis_focus",
                "summary, frequent words, terminology, reading advice",
            ),
        ],
    );

    Ok((
        StatusCode::OK,
        Json(ApiResponse::new(DocumentAnalysisResponse {
            document_id,
            title: title.to_string(),
            summary,
            frequent_words,
            terminology,
            suggestions,
            prompt,
            provider: "local-deterministic",
        })),
    ))
}

fn prompt_info(name: &'static str, values: &[(&str, &str)]) -> PromptInfo {
    let mut template = match name {
        "selection_translate" => {
            "Analyze selected academic text for translation, terms, and sentence structure.".to_string()
        }
        "document_summary" => {
            "Analyze the document for summary, frequent vocabulary, terminology, and reading advice."
                .to_string()
        }
        _ => "Analyze the reading material.".to_string(),
    };

    for (key, value) in values {
        template.push_str(&format!("\n{key}: {value}"));
    }

    PromptInfo { name, template }
}

fn translate_selection(text: &str, terms: &[TermInfo], target_language: &str) -> String {
    let mut hints: Vec<String> = terms
        .iter()
        .take(4)
        .map(|term| format!("{} 可理解为{}", term.term, term.explanation))
        .collect();

    if hints.is_empty() {
        hints.push("可先按主干意思理解，再回到上下文确认细节".to_string());
    }

    format!(
        "面向 {target_language} 的辅助翻译：这段话的核心意思是“{}”。术语提示：{}。",
        compact_preview(text, 80),
        hints.join("；")
    )
}

fn build_selection_analysis(terms: &[TermInfo], sentence_analysis: &SentenceAnalysis) -> String {
    let term_note = if terms.is_empty() {
        "未识别到明显专业术语".to_string()
    } else {
        format!("识别到 {} 个专业术语", terms.len())
    };

    format!(
        "{}；句子难度为{}。建议先抓主句“{}”，再处理连接词后的补充信息。",
        term_note,
        sentence_analysis.difficulty,
        compact_preview(&sentence_analysis.main_clause, 48)
    )
}

fn analyze_sentence(text: &str, term_count: usize) -> SentenceAnalysis {
    let clauses = split_clauses(text);
    let main_clause = clauses.first().cloned().unwrap_or_else(|| text.to_string());
    let punctuation_count = text
        .chars()
        .filter(|character| matches!(character, ',' | ';' | ':' | '(' | ')'))
        .count();
    let difficulty = if text.chars().count() > 160 || clauses.len() >= 4 || term_count >= 4 {
        "high"
    } else if text.chars().count() > 90 || punctuation_count >= 2 || clauses.len() >= 2 {
        "medium"
    } else {
        "low"
    }
    .to_string();

    let analyzed_clauses = clauses
        .iter()
        .enumerate()
        .map(|(index, clause)| ClauseAnalysis {
            role: if index == 0 {
                "main".to_string()
            } else {
                "supporting".to_string()
            },
            text: clause.clone(),
            note: if index == 0 {
                "优先定位主语、谓语和核心动作。".to_string()
            } else {
                "作为原因、条件、转折或补充信息处理。".to_string()
            },
        })
        .collect();

    SentenceAnalysis {
        difficulty,
        main_clause,
        clauses: analyzed_clauses,
        strategy: vec![
            "先找主句，暂时跳过插入语和修饰成分。".to_string(),
            "把 because, although, which, that 等连接词后的内容作为层级补充。".to_string(),
            "遇到术语先保留英文，再结合上下文确认中文含义。".to_string(),
        ],
    }
}

fn split_clauses(text: &str) -> Vec<String> {
    let normalized = text
        .replace(" because ", ", because ")
        .replace(" although ", ", although ")
        .replace(" while ", ", while ")
        .replace(" which ", ", which ")
        .replace(" that ", ", that ");

    normalized
        .split(|character| matches!(character, ',' | ';' | ':'))
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn detect_terms(text: &str) -> Vec<TermInfo> {
    let lower = text.to_lowercase();
    let tokens = tokenize(&lower);
    let mut token_counts: HashMap<&str, usize> = HashMap::new();
    for token in &tokens {
        *token_counts.entry(token.as_str()).or_insert(0) += 1;
    }

    let glossary = [
        ("algorithm", "computer science", "算法"),
        ("neural network", "machine learning", "神经网络"),
        ("dataset", "data science", "数据集"),
        ("performance", "academic metric", "性能或表现"),
        ("evaluation", "research method", "评估"),
        ("model", "machine learning", "模型"),
        ("feature", "machine learning", "特征"),
        ("translation", "language learning", "翻译"),
        ("analysis", "academic reading", "分析"),
        ("terminology", "academic reading", "专业术语"),
    ];

    let mut terms = Vec::new();
    let mut seen = HashSet::new();

    for (term, category, explanation) in glossary {
        let frequency = if term.contains(' ') {
            lower.matches(term).count()
        } else {
            token_counts.get(term).copied().unwrap_or(0)
        };

        if frequency > 0 {
            seen.insert(term.to_string());
            terms.push(TermInfo {
                term: term.to_string(),
                category: category.to_string(),
                explanation: explanation.to_string(),
                frequency,
            });
        }
    }

    for (token, frequency) in token_counts {
        if frequency >= 2 && token.len() >= 8 && !seen.contains(token) {
            terms.push(TermInfo {
                term: token.to_string(),
                category: "repeated keyword".to_string(),
                explanation: "文中重复出现的高信息密度词".to_string(),
                frequency,
            });
        }
    }

    terms.sort_by(|a, b| {
        b.frequency
            .cmp(&a.frequency)
            .then_with(|| a.term.cmp(&b.term))
    });
    terms
}

fn frequent_words(text: &str, limit: usize) -> Vec<FrequentWord> {
    let stop_words = stop_words();
    let tokens = tokenize(text);
    let mut counts: HashMap<String, usize> = HashMap::new();

    for token in tokens {
        if token.len() < 4 || stop_words.contains(token.as_str()) {
            continue;
        }
        *counts.entry(token).or_insert(0) += 1;
    }

    let mut words: Vec<FrequentWord> = counts
        .into_iter()
        .map(|(word, count)| FrequentWord { word, count })
        .collect();
    words.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.word.cmp(&b.word)));
    words.truncate(limit);
    words
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|character: char| !character.is_ascii_alphabetic())
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn stop_words() -> HashSet<&'static str> {
    [
        "about", "after", "also", "because", "been", "between", "from", "into", "over", "that",
        "their", "then", "there", "these", "this", "those", "through", "with", "within",
    ]
    .into_iter()
    .collect()
}

fn build_document_summary(
    title: &str,
    paragraph_count: usize,
    frequent_words: &[FrequentWord],
    terminology: &[TermInfo],
) -> String {
    let top_words = frequent_words
        .iter()
        .take(3)
        .map(|word| word.word.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let top_terms = terminology
        .iter()
        .take(3)
        .map(|term| term.term.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "{title} 包含 {paragraph_count} 个段落，核心高频词包括 {top_words}，重点术语包括 {top_terms}。建议围绕这些词建立阅读提纲。"
    )
}

fn build_document_suggestions(
    frequent_words: &[FrequentWord],
    terminology: &[TermInfo],
) -> Vec<String> {
    let mut suggestions = vec![
        "先快速浏览标题和段首句，建立主题框架。".to_string(),
        "把高频词整理成复习词表，并回到原文确认搭配。".to_string(),
    ];

    if !terminology.is_empty() {
        suggestions.push("专业术语建议保留英文原词，旁边记录中文解释和例句。".to_string());
    }

    if frequent_words.iter().any(|word| word.count >= 3) {
        suggestions.push("重复出现三次以上的词可作为文献主线关键词。".to_string());
    }

    suggestions
}

fn compact_preview(text: &str, limit: usize) -> String {
    let mut chars = text.chars();
    let preview: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        format!("{preview}...")
    } else {
        preview
    }
}
