use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    error::AppError,
    response::{ApiJson, ApiResponse},
    state::AppState,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProgressRequest {
    #[validate(range(min = 0))]
    pub paragraph_position: i64,
    #[validate(range(min = 0.0, max = 100.0))]
    pub progress_percent: f64,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct ReadingProgress {
    pub document_id: String,
    pub paragraph_position: i64,
    pub progress_percent: f64,
    pub updated_at: String,
}

#[utoipa::path(put, path = "/api/v1/documents/{id}/progress", params(("id" = String, Path)), request_body = UpdateProgressRequest, security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<ReadingProgress>), (status = 404, description = "Document not found or owned by another user")))]
pub async fn update(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
    ApiJson(input): ApiJson<UpdateProgressRequest>,
) -> Result<Json<ApiResponse<ReadingProgress>>, AppError> {
    input
        .validate()
        .map_err(|error| AppError::Validation(error.to_string()))?;
    let paragraph_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM document_paragraphs p JOIN documents d ON d.id = p.document_id WHERE d.id = ? AND d.user_id = ?")
        .bind(&document_id).bind(&user_id).fetch_one(&state.db).await?;
    if paragraph_count == 0 {
        return Err(AppError::NotFound);
    }
    if input.paragraph_position >= paragraph_count {
        return Err(AppError::Validation(
            "paragraph_position is outside the document".into(),
        ));
    }
    let now = Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO reading_progress (id, user_id, document_id, paragraph_position, progress_percent, updated_at) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(user_id, document_id) DO UPDATE SET paragraph_position = excluded.paragraph_position, progress_percent = excluded.progress_percent, updated_at = excluded.updated_at")
        .bind(Uuid::new_v4().to_string()).bind(&user_id).bind(&document_id).bind(input.paragraph_position).bind(input.progress_percent).bind(&now).execute(&state.db).await?;
    Ok(Json(ApiResponse::new(ReadingProgress {
        document_id,
        paragraph_position: input.paragraph_position,
        progress_percent: input.progress_percent,
        updated_at: now,
    })))
}

#[utoipa::path(get, path = "/api/v1/documents/{id}/progress", params(("id" = String, Path)), security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Option<ReadingProgress>>), (status = 401, body = crate::response::ErrorBody), (status = 404, body = crate::response::ErrorBody)))]
pub async fn get(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> Result<Json<ApiResponse<Option<ReadingProgress>>>, AppError> {
    let document_exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE id = ? AND user_id = ?")
            .bind(&document_id)
            .bind(&user_id)
            .fetch_one(&state.db)
            .await?;
    if document_exists == 0 {
        return Err(AppError::NotFound);
    }
    let progress = sqlx::query_as::<_, ReadingProgress>(
        "SELECT document_id, paragraph_position, progress_percent, updated_at FROM reading_progress WHERE document_id = ? AND user_id = ?",
    )
    .bind(document_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;
    Ok(Json(ApiResponse::new(progress)))
}
