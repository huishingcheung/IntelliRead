use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::AppError,
    response::{ApiJson, ApiResponse},
    state::AppState,
};

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct Note {
    pub id: String,
    pub document_id: String,
    pub paragraph_id: Option<String>,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNoteRequest {
    pub paragraph_id: Option<String>,
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNoteRequest {
    pub content: String,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct Highlight {
    pub id: String,
    pub document_id: String,
    pub paragraph_id: String,
    pub start_offset: i64,
    pub end_offset: i64,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateHighlightRequest {
    pub paragraph_id: String,
    pub start_offset: i64,
    pub end_offset: i64,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateHighlightRequest {
    pub start_offset: Option<i64>,
    pub end_offset: Option<i64>,
    pub color: Option<String>,
}

#[utoipa::path(post, path = "/api/v1/documents/{id}/notes", params(("id" = String, Path)), request_body = CreateNoteRequest, security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Note>)))]
pub async fn create_note(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
    ApiJson(input): ApiJson<CreateNoteRequest>,
) -> Result<Json<ApiResponse<Note>>, AppError> {
    validate_content(&input.content)?;
    ensure_document(&state, &user_id, &document_id).await?;
    if let Some(paragraph_id) = input.paragraph_id.as_deref() {
        paragraph_content(&state, &user_id, &document_id, paragraph_id).await?;
    }
    let now = Utc::now().to_rfc3339();
    let note = Note {
        id: Uuid::new_v4().to_string(),
        document_id,
        paragraph_id: input.paragraph_id,
        content: input.content.trim().to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    sqlx::query("INSERT INTO notes (id, user_id, document_id, paragraph_id, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&note.id).bind(&user_id).bind(&note.document_id).bind(&note.paragraph_id).bind(&note.content).bind(&note.created_at).bind(&note.updated_at).execute(&state.db).await?;
    Ok(Json(ApiResponse::new(note)))
}

#[utoipa::path(get, path = "/api/v1/documents/{id}/notes", params(("id" = String, Path)), security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Vec<Note>>)))]
pub async fn list_notes(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Note>>>, AppError> {
    ensure_document(&state, &user_id, &document_id).await?;
    let notes = sqlx::query_as::<_, Note>("SELECT id, document_id, paragraph_id, content, created_at, updated_at FROM notes WHERE user_id = ? AND document_id = ? ORDER BY created_at DESC")
        .bind(user_id).bind(document_id).fetch_all(&state.db).await?;
    Ok(Json(ApiResponse::new(notes)))
}

#[utoipa::path(put, path = "/api/v1/notes/{id}", params(("id" = String, Path)), request_body = UpdateNoteRequest, security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Note>)))]
pub async fn update_note(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    ApiJson(input): ApiJson<UpdateNoteRequest>,
) -> Result<Json<ApiResponse<Note>>, AppError> {
    validate_content(&input.content)?;
    let now = Utc::now().to_rfc3339();
    let result =
        sqlx::query("UPDATE notes SET content = ?, updated_at = ? WHERE id = ? AND user_id = ?")
            .bind(input.content.trim())
            .bind(&now)
            .bind(&id)
            .bind(&user_id)
            .execute(&state.db)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    let note = sqlx::query_as::<_, Note>("SELECT id, document_id, paragraph_id, content, created_at, updated_at FROM notes WHERE id = ? AND user_id = ?")
        .bind(id).bind(user_id).fetch_one(&state.db).await?;
    Ok(Json(ApiResponse::new(note)))
}

#[utoipa::path(delete, path = "/api/v1/notes/{id}", params(("id" = String, Path)), security(("bearer_auth" = [])), responses((status = 204, description = "Note deleted")))]
pub async fn delete_note(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    delete_owned(&state, "notes", &id, &user_id).await
}

#[utoipa::path(post, path = "/api/v1/documents/{id}/highlights", params(("id" = String, Path)), request_body = CreateHighlightRequest, security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Highlight>)))]
pub async fn create_highlight(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
    ApiJson(input): ApiJson<CreateHighlightRequest>,
) -> Result<Json<ApiResponse<Highlight>>, AppError> {
    let content = paragraph_content(&state, &user_id, &document_id, &input.paragraph_id).await?;
    validate_range(&content, input.start_offset, input.end_offset)?;
    let color = validate_color(input.color.as_deref().unwrap_or("yellow"))?;
    let now = Utc::now().to_rfc3339();
    let highlight = Highlight {
        id: Uuid::new_v4().to_string(),
        document_id,
        paragraph_id: input.paragraph_id,
        start_offset: input.start_offset,
        end_offset: input.end_offset,
        color: color.to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    sqlx::query("INSERT INTO highlights (id, user_id, document_id, paragraph_id, start_offset, end_offset, color, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&highlight.id).bind(&user_id).bind(&highlight.document_id).bind(&highlight.paragraph_id).bind(highlight.start_offset).bind(highlight.end_offset).bind(&highlight.color).bind(&highlight.created_at).bind(&highlight.updated_at).execute(&state.db).await?;
    Ok(Json(ApiResponse::new(highlight)))
}

#[utoipa::path(get, path = "/api/v1/documents/{id}/highlights", params(("id" = String, Path)), security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Vec<Highlight>>)))]
pub async fn list_highlights(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Highlight>>>, AppError> {
    ensure_document(&state, &user_id, &document_id).await?;
    let highlights = sqlx::query_as::<_, Highlight>("SELECT id, document_id, paragraph_id, start_offset, end_offset, color, created_at, updated_at FROM highlights WHERE user_id = ? AND document_id = ? ORDER BY created_at DESC")
        .bind(user_id).bind(document_id).fetch_all(&state.db).await?;
    Ok(Json(ApiResponse::new(highlights)))
}

#[utoipa::path(put, path = "/api/v1/highlights/{id}", params(("id" = String, Path)), request_body = UpdateHighlightRequest, security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Highlight>)))]
pub async fn update_highlight(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    ApiJson(input): ApiJson<UpdateHighlightRequest>,
) -> Result<Json<ApiResponse<Highlight>>, AppError> {
    let current = sqlx::query_as::<_, Highlight>("SELECT id, document_id, paragraph_id, start_offset, end_offset, color, created_at, updated_at FROM highlights WHERE id = ? AND user_id = ?")
        .bind(&id).bind(&user_id).fetch_optional(&state.db).await?.ok_or(AppError::NotFound)?;
    let start = input.start_offset.unwrap_or(current.start_offset);
    let end = input.end_offset.unwrap_or(current.end_offset);
    let content = paragraph_content(
        &state,
        &user_id,
        &current.document_id,
        &current.paragraph_id,
    )
    .await?;
    validate_range(&content, start, end)?;
    let color = validate_color(input.color.as_deref().unwrap_or(&current.color))?;
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE highlights SET start_offset = ?, end_offset = ?, color = ?, updated_at = ? WHERE id = ? AND user_id = ?")
        .bind(start).bind(end).bind(color).bind(&now).bind(&id).bind(&user_id).execute(&state.db).await?;
    Ok(Json(ApiResponse::new(Highlight {
        start_offset: start,
        end_offset: end,
        color: color.to_string(),
        updated_at: now,
        ..current
    })))
}

#[utoipa::path(delete, path = "/api/v1/highlights/{id}", params(("id" = String, Path)), security(("bearer_auth" = [])), responses((status = 204, description = "Highlight deleted")))]
pub async fn delete_highlight(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    delete_owned(&state, "highlights", &id, &user_id).await
}

async fn delete_owned(
    state: &AppState,
    table: &str,
    id: &str,
    user_id: &str,
) -> Result<StatusCode, AppError> {
    let query = match table {
        "notes" => "DELETE FROM notes WHERE id = ? AND user_id = ?",
        "highlights" => "DELETE FROM highlights WHERE id = ? AND user_id = ?",
        _ => return Err(AppError::Internal),
    };
    let result = sqlx::query(query)
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn ensure_document(
    state: &AppState,
    user_id: &str,
    document_id: &str,
) -> Result<(), AppError> {
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

async fn paragraph_content(
    state: &AppState,
    user_id: &str,
    document_id: &str,
    paragraph_id: &str,
) -> Result<String, AppError> {
    sqlx::query_scalar("SELECT p.content FROM document_paragraphs p JOIN documents d ON d.id = p.document_id WHERE p.id = ? AND p.document_id = ? AND d.user_id = ?")
        .bind(paragraph_id).bind(document_id).bind(user_id).fetch_optional(&state.db).await?.ok_or(AppError::NotFound)
}

fn validate_content(content: &str) -> Result<(), AppError> {
    let length = content.trim().chars().count();
    if length == 0 || length > 10_000 {
        return Err(AppError::Validation(
            "note content must contain 1 to 10000 characters".into(),
        ));
    }
    Ok(())
}

fn validate_range(content: &str, start: i64, end: i64) -> Result<(), AppError> {
    let length = content.chars().count() as i64;
    if start < 0 || end <= start || end > length {
        return Err(AppError::Validation(
            "highlight range is outside the paragraph".into(),
        ));
    }
    Ok(())
}

fn validate_color(color: &str) -> Result<&str, AppError> {
    match color {
        "yellow" | "green" | "blue" | "pink" | "purple" => Ok(color),
        _ => Err(AppError::Validation("unsupported highlight color".into())),
    }
}
