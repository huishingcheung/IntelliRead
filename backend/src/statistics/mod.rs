use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{auth::AuthUser, error::AppError, response::ApiResponse, state::AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct LearningOverview {
    pub active_documents: i64,
    pub archived_documents: i64,
    pub paragraphs: i64,
    pub tags: i64,
    pub notes: i64,
    pub highlights: i64,
    pub tracked_documents: i64,
    pub average_progress_percent: f64,
}

#[utoipa::path(get, path = "/api/v1/statistics/overview", security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<LearningOverview>)))]
pub async fn overview(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<LearningOverview>>, AppError> {
    let active_documents = count(
        &state,
        "SELECT COUNT(*) FROM documents WHERE user_id = ? AND archived_at IS NULL",
        &user_id,
    )
    .await?;
    let archived_documents = count(
        &state,
        "SELECT COUNT(*) FROM documents WHERE user_id = ? AND archived_at IS NOT NULL",
        &user_id,
    )
    .await?;
    let paragraphs = count(
        &state,
        "SELECT COUNT(*) FROM document_paragraphs p JOIN documents d ON d.id = p.document_id WHERE d.user_id = ?",
        &user_id,
    )
    .await?;
    let tags = count(
        &state,
        "SELECT COUNT(*) FROM tags WHERE user_id = ?",
        &user_id,
    )
    .await?;
    let notes = count(
        &state,
        "SELECT COUNT(*) FROM notes WHERE user_id = ?",
        &user_id,
    )
    .await?;
    let highlights = count(
        &state,
        "SELECT COUNT(*) FROM highlights WHERE user_id = ?",
        &user_id,
    )
    .await?;
    let tracked_documents = count(
        &state,
        "SELECT COUNT(*) FROM reading_progress WHERE user_id = ?",
        &user_id,
    )
    .await?;
    let average_progress_percent: f64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(progress_percent), 0.0) FROM reading_progress WHERE user_id = ?",
    )
    .bind(&user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse::new(LearningOverview {
        active_documents,
        archived_documents,
        paragraphs,
        tags,
        notes,
        highlights,
        tracked_documents,
        average_progress_percent,
    })))
}

async fn count(state: &AppState, query: &str, user_id: &str) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar(query)
        .bind(user_id)
        .fetch_one(&state.db)
        .await?)
}
