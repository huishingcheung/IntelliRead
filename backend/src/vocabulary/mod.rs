use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::AppError,
    response::{ApiJson, ApiResponse},
    state::AppState,
};

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct VocabularyCard {
    pub id: String,
    pub document_id: String,
    pub paragraph_id: Option<String>,
    pub term: String,
    pub pronunciation: Option<String>,
    pub definition: String,
    pub example_sentence: Option<String>,
    pub source_text: Option<String>,
    pub mastery_status: String,
    pub next_review_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct ReviewAnswer {
    pub id: String,
    pub vocabulary_id: String,
    pub answer_result: String,
    pub reviewed_at: String,
    pub next_review_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VocabularyList {
    pub items: Vec<VocabularyCard>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateVocabularyRequest {
    pub document_id: String,
    pub paragraph_id: Option<String>,
    pub term: String,
    pub pronunciation: Option<String>,
    pub definition: String,
    pub example_sentence: Option<String>,
    pub source_text: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateVocabularyRequest {
    pub definition: Option<String>,
    pub example_sentence: Option<String>,
    pub mastery_status: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct VocabularyQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub mastery_status: Option<String>,
    pub document_id: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ReviewQueueQuery {
    pub limit: Option<i64>,
    pub document_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewAnswerRequest {
    pub vocabulary_id: String,
    pub answer_result: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/vocabulary",
    params(VocabularyQuery),
    security(("bearer_auth" = [])),
    responses((status = 200, body = ApiResponse<VocabularyList>))
)]
pub async fn list(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<VocabularyQuery>,
) -> Result<Json<ApiResponse<VocabularyList>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;
    let sort = match query.sort.as_deref().unwrap_or("created_at") {
        "created_at" => "created_at",
        "next_review_at" => "next_review_at",
        "term" => "term",
        _ => return Err(AppError::Validation("invalid sort field".into())),
    };
    let order = match query.order.as_deref().unwrap_or("desc") {
        "asc" => "ASC",
        "desc" => "DESC",
        _ => return Err(AppError::Validation("invalid order".into())),
    };
    if let Some(status) = &query.mastery_status {
        validate_mastery_status(status)?;
    }

    let mut where_sql = String::from("WHERE user_id = ?");
    if query.mastery_status.is_some() {
        where_sql.push_str(" AND mastery_status = ?");
    }
    if query.document_id.is_some() {
        where_sql.push_str(" AND document_id = ?");
    }

    let count_sql = format!("SELECT COUNT(*) FROM vocabulary_cards {where_sql}");
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql).bind(&user_id);
    if let Some(status) = &query.mastery_status {
        count_query = count_query.bind(status);
    }
    if let Some(document_id) = &query.document_id {
        count_query = count_query.bind(document_id);
    }
    let total = count_query.fetch_one(&state.db).await?;

    let list_sql = format!(
        "SELECT id, document_id, paragraph_id, term, pronunciation, definition, example_sentence, source_text, mastery_status, next_review_at, created_at, updated_at FROM vocabulary_cards {where_sql} ORDER BY {sort} {order} LIMIT ? OFFSET ?"
    );
    let mut list_query = sqlx::query_as::<_, VocabularyCard>(&list_sql).bind(&user_id);
    if let Some(status) = &query.mastery_status {
        list_query = list_query.bind(status);
    }
    if let Some(document_id) = &query.document_id {
        list_query = list_query.bind(document_id);
    }
    let items = list_query
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(ApiResponse::new(VocabularyList {
        items,
        page,
        page_size,
        total,
    })))
}

#[utoipa::path(
    post,
    path = "/api/v1/vocabulary",
    request_body = CreateVocabularyRequest,
    security(("bearer_auth" = [])),
    responses((status = 200, body = ApiResponse<VocabularyCard>), (status = 409, description = "Vocabulary already exists"))
)]
pub async fn create(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    ApiJson(input): ApiJson<CreateVocabularyRequest>,
) -> Result<Json<ApiResponse<VocabularyCard>>, AppError> {
    ensure_document(&state, &user_id, &input.document_id).await?;
    if let Some(paragraph_id) = &input.paragraph_id {
        ensure_paragraph(&state, &input.document_id, paragraph_id).await?;
    }

    let term = validate_required_text(&input.term, "term")?;
    let definition = validate_required_text(&input.definition, "definition")?;
    let now = Utc::now().to_rfc3339();
    let card = VocabularyCard {
        id: Uuid::new_v4().to_string(),
        document_id: input.document_id,
        paragraph_id: input.paragraph_id,
        term: term.to_string(),
        pronunciation: normalize_optional(input.pronunciation),
        definition: definition.to_string(),
        example_sentence: normalize_optional(input.example_sentence),
        source_text: normalize_optional(input.source_text),
        mastery_status: "new".into(),
        next_review_at: None,
        created_at: now.clone(),
        updated_at: now,
    };

    let result = sqlx::query(
        "INSERT INTO vocabulary_cards (id, user_id, document_id, paragraph_id, term, pronunciation, definition, example_sentence, source_text, mastery_status, next_review_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&card.id)
    .bind(&user_id)
    .bind(&card.document_id)
    .bind(&card.paragraph_id)
    .bind(&card.term)
    .bind(&card.pronunciation)
    .bind(&card.definition)
    .bind(&card.example_sentence)
    .bind(&card.source_text)
    .bind(&card.mastery_status)
    .bind(&card.next_review_at)
    .bind(&card.created_at)
    .bind(&card.updated_at)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Ok(Json(ApiResponse::new(card))),
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
            Err(AppError::Conflict("vocabulary already exists".into()))
        }
        Err(error) => Err(error.into()),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/vocabulary/{id}",
    params(("id" = String, Path)),
    security(("bearer_auth" = [])),
    responses((status = 200, body = ApiResponse<VocabularyCard>), (status = 404, description = "Vocabulary not found"))
)]
pub async fn detail(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<VocabularyCard>>, AppError> {
    let card = get_card(&state, &user_id, &id).await?;
    Ok(Json(ApiResponse::new(card)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/vocabulary/{id}",
    params(("id" = String, Path)),
    request_body = UpdateVocabularyRequest,
    security(("bearer_auth" = [])),
    responses((status = 200, body = ApiResponse<VocabularyCard>), (status = 404, description = "Vocabulary not found"))
)]
pub async fn update(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    ApiJson(input): ApiJson<UpdateVocabularyRequest>,
) -> Result<Json<ApiResponse<VocabularyCard>>, AppError> {
    let mut card = get_card(&state, &user_id, &id).await?;
    if let Some(definition) = input.definition {
        card.definition = validate_required_text(&definition, "definition")?.to_string();
    }
    if let Some(example_sentence) = input.example_sentence {
        card.example_sentence = normalize_optional(Some(example_sentence));
    }
    if let Some(status) = input.mastery_status {
        validate_mastery_status(&status)?;
        card.mastery_status = status;
    }
    card.updated_at = Utc::now().to_rfc3339();

    sqlx::query("UPDATE vocabulary_cards SET definition = ?, example_sentence = ?, mastery_status = ?, updated_at = ? WHERE id = ? AND user_id = ?")
        .bind(&card.definition)
        .bind(&card.example_sentence)
        .bind(&card.mastery_status)
        .bind(&card.updated_at)
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;

    Ok(Json(ApiResponse::new(card)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/vocabulary/{id}",
    params(("id" = String, Path)),
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Vocabulary deleted"))
)]
pub async fn delete(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM vocabulary_cards WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/review/queue",
    params(ReviewQueueQuery),
    security(("bearer_auth" = [])),
    responses((status = 200, body = ApiResponse<Vec<VocabularyCard>>))
)]
pub async fn review_queue(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReviewQueueQuery>,
) -> Result<Json<ApiResponse<Vec<VocabularyCard>>>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let now = Utc::now().to_rfc3339();

    let cards = if let Some(document_id) = query.document_id {
        sqlx::query_as::<_, VocabularyCard>(
            "SELECT id, document_id, paragraph_id, term, pronunciation, definition, example_sentence, source_text, mastery_status, next_review_at, created_at, updated_at FROM vocabulary_cards WHERE user_id = ? AND document_id = ? AND mastery_status != 'mastered' AND (next_review_at IS NULL OR next_review_at <= ?) ORDER BY next_review_at ASC LIMIT ?",
        )
        .bind(user_id)
        .bind(document_id)
        .bind(now)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, VocabularyCard>(
            "SELECT id, document_id, paragraph_id, term, pronunciation, definition, example_sentence, source_text, mastery_status, next_review_at, created_at, updated_at FROM vocabulary_cards WHERE user_id = ? AND mastery_status != 'mastered' AND (next_review_at IS NULL OR next_review_at <= ?) ORDER BY next_review_at ASC LIMIT ?",
        )
        .bind(user_id)
        .bind(now)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(ApiResponse::new(cards)))
}

#[utoipa::path(
    post,
    path = "/api/v1/review/answer",
    request_body = ReviewAnswerRequest,
    security(("bearer_auth" = [])),
    responses((status = 200, body = ApiResponse<ReviewAnswer>), (status = 404, description = "Vocabulary not found"))
)]
pub async fn submit_review_answer(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    ApiJson(input): ApiJson<ReviewAnswerRequest>,
) -> Result<Json<ApiResponse<ReviewAnswer>>, AppError> {
    validate_answer_result(&input.answer_result)?;
    get_card(&state, &user_id, &input.vocabulary_id).await?;

    let reviewed_at = Utc::now();
    let (mastery_status, next_review_at) = schedule_review(&input.answer_result, reviewed_at);
    let answer = ReviewAnswer {
        id: Uuid::new_v4().to_string(),
        vocabulary_id: input.vocabulary_id,
        answer_result: input.answer_result,
        reviewed_at: reviewed_at.to_rfc3339(),
        next_review_at: next_review_at.to_rfc3339(),
    };

    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO review_answers (id, user_id, vocabulary_id, answer_result, reviewed_at, next_review_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&answer.id)
        .bind(&user_id)
        .bind(&answer.vocabulary_id)
        .bind(&answer.answer_result)
        .bind(&answer.reviewed_at)
        .bind(&answer.next_review_at)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE vocabulary_cards SET mastery_status = ?, next_review_at = ?, updated_at = ? WHERE id = ? AND user_id = ?")
        .bind(mastery_status)
        .bind(&answer.next_review_at)
        .bind(&answer.reviewed_at)
        .bind(&answer.vocabulary_id)
        .bind(&user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(ApiResponse::new(answer)))
}

async fn get_card(state: &AppState, user_id: &str, id: &str) -> Result<VocabularyCard, AppError> {
    sqlx::query_as::<_, VocabularyCard>(
        "SELECT id, document_id, paragraph_id, term, pronunciation, definition, example_sentence, source_text, mastery_status, next_review_at, created_at, updated_at FROM vocabulary_cards WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)
}

async fn ensure_document(state: &AppState, user_id: &str, document_id: &str) -> Result<(), AppError> {
    let exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE id = ? AND user_id = ?")
            .bind(document_id)
            .bind(user_id)
            .fetch_one(&state.db)
            .await?;
    if exists == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(())
    }
}

async fn ensure_paragraph(
    state: &AppState,
    document_id: &str,
    paragraph_id: &str,
) -> Result<(), AppError> {
    let exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM document_paragraphs WHERE id = ? AND document_id = ?")
            .bind(paragraph_id)
            .bind(document_id)
            .fetch_one(&state.db)
            .await?;
    if exists == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(())
    }
}

fn validate_required_text<'a>(value: &'a str, field: &str) -> Result<&'a str, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field} is required")));
    }
    Ok(value)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn validate_mastery_status(status: &str) -> Result<(), AppError> {
    match status {
        "new" | "learning" | "familiar" | "mastered" => Ok(()),
        _ => Err(AppError::Validation("invalid mastery_status".into())),
    }
}

fn validate_answer_result(result: &str) -> Result<(), AppError> {
    match result {
        "wrong" | "hard" | "good" | "easy" => Ok(()),
        _ => Err(AppError::Validation("invalid answer_result".into())),
    }
}

fn schedule_review(result: &str, now: chrono::DateTime<Utc>) -> (&'static str, chrono::DateTime<Utc>) {
    match result {
        "wrong" => ("learning", now + Duration::minutes(10)),
        "hard" => ("learning", now + Duration::days(1)),
        "good" => ("familiar", now + Duration::days(3)),
        "easy" => ("mastered", now + Duration::days(7)),
        _ => unreachable!("answer_result must be validated before scheduling"),
    }
}