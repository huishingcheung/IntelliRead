use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::AppError,
    response::{ApiJson, ApiMultipart, ApiQuery, ApiResponse},
    state::AppState,
};

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct DocumentSummary {
    pub id: String,
    pub title: String,
    pub source_type: String,
    pub original_filename: String,
    pub byte_size: i64,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct Paragraph {
    pub id: String,
    pub position: i64,
    pub content: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentDetail {
    #[serde(flatten)]
    pub document: DocumentSummary,
    pub paragraphs: Vec<Paragraph>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListQuery {
    pub q: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub archived: Option<bool>,
    pub tag_id: Option<String>,
}

#[derive(Debug, ToSchema)]
pub struct ImportDocumentForm {
    pub title: Option<String>,
    #[schema(format = Binary)]
    pub file: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub archived: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentList {
    pub items: Vec<DocumentSummary>,
    pub limit: u32,
    pub offset: u32,
}

#[utoipa::path(post, path = "/api/v1/documents", request_body(content = ImportDocumentForm, content_type = "multipart/form-data"), security(("bearer_auth" = [])), responses((status = 201, body = ApiResponse<DocumentDetail>), (status = 400, body = crate::response::ErrorBody), (status = 401, body = crate::response::ErrorBody), (status = 413, body = crate::response::ErrorBody), (status = 415, body = crate::response::ErrorBody)))]
pub async fn import(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    ApiMultipart(mut multipart): ApiMultipart,
) -> Result<(StatusCode, Json<ApiResponse<DocumentDetail>>), AppError> {
    let mut title = None;
    let mut file = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Validation("invalid multipart body".into()))?
    {
        match field.name() {
            Some("title") => {
                title = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| AppError::Validation("invalid title".into()))?,
                )
            }
            Some("file") => {
                let filename = field.file_name().unwrap_or("document.txt").to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::Validation("invalid file".into()))?;
                file = Some((filename, bytes));
            }
            _ => {}
        }
    }
    let (filename, bytes) = file.ok_or_else(|| AppError::Validation("file is required".into()))?;
    if bytes.len() > state.config.max_document_bytes {
        return Err(AppError::PayloadTooLarge);
    }
    let source_type = match filename
        .rsplit('.')
        .next()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("md") | Some("markdown") => "markdown",
        Some("txt") => "txt",
        _ => {
            return Err(AppError::UnsupportedMediaType(
                "only .md, .markdown and .txt files are accepted".into(),
            ));
        }
    };
    let content = std::str::from_utf8(&bytes)
        .map_err(|_| AppError::Validation("file must be UTF-8 encoded".into()))?;
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let paragraphs = split_paragraphs(content);
    if paragraphs.is_empty() {
        return Err(AppError::Validation(
            "document contains no readable text".into(),
        ));
    }
    let title = title.filter(|v| !v.trim().is_empty()).unwrap_or_else(|| {
        filename
            .rsplit_once('.')
            .map(|(stem, _)| stem)
            .unwrap_or(&filename)
            .to_string()
    });
    if title.chars().count() > 200 {
        return Err(AppError::Validation(
            "title must not exceed 200 characters".into(),
        ));
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO documents (id, user_id, title, source_type, original_filename, byte_size, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&user_id).bind(title.trim()).bind(source_type).bind(&filename).bind(bytes.len() as i64).bind(&now).bind(&now).execute(&mut *tx).await?;
    for (position, content) in paragraphs.iter().enumerate() {
        sqlx::query("INSERT INTO document_paragraphs (id, document_id, position, content, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string()).bind(&id).bind(position as i64).bind(content).bind(&now).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    get_document(&state, &user_id, &id)
        .await
        .map(|detail| (StatusCode::CREATED, Json(ApiResponse::new(detail))))
}

#[utoipa::path(get, path = "/api/v1/documents", params(ListQuery), security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<DocumentList>), (status = 400, body = crate::response::ErrorBody), (status = 401, body = crate::response::ErrorBody)))]
pub async fn list(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    ApiQuery(query): ApiQuery<ListQuery>,
) -> Result<Json<ApiResponse<DocumentList>>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);
    let pattern = format!("%{}%", query.q.unwrap_or_default().trim());
    let archived = query.archived.unwrap_or(false);
    let tag_id = query.tag_id.unwrap_or_default();
    let items = sqlx::query_as::<_, DocumentSummary>("SELECT id, title, source_type, original_filename, byte_size, created_at, updated_at, archived_at FROM documents WHERE user_id = ? AND ((? = 1 AND archived_at IS NOT NULL) OR (? = 0 AND archived_at IS NULL)) AND (title LIKE ? OR EXISTS (SELECT 1 FROM document_paragraphs p WHERE p.document_id = documents.id AND p.content LIKE ?)) AND (? = '' OR EXISTS (SELECT 1 FROM document_tags dt JOIN tags t ON t.id = dt.tag_id WHERE dt.document_id = documents.id AND t.id = ? AND t.user_id = documents.user_id)) ORDER BY created_at DESC LIMIT ? OFFSET ?")
        .bind(user_id).bind(archived).bind(archived).bind(&pattern).bind(&pattern).bind(&tag_id).bind(&tag_id).bind(limit).bind(offset).fetch_all(&state.db).await?;
    Ok(Json(ApiResponse::new(DocumentList {
        items,
        limit,
        offset,
    })))
}

#[utoipa::path(patch, path = "/api/v1/documents/{id}", params(("id" = String, Path)), request_body = UpdateDocumentRequest, security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<DocumentDetail>), (status = 404, description = "Document not found or owned by another user")))]
pub async fn update(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    ApiJson(input): ApiJson<UpdateDocumentRequest>,
) -> Result<Json<ApiResponse<DocumentDetail>>, AppError> {
    let title = input.title.as_deref().map(str::trim);
    if title.is_some_and(|value| value.is_empty() || value.chars().count() > 200) {
        return Err(AppError::Validation(
            "title must contain 1 to 200 characters".into(),
        ));
    }
    if title.is_none() && input.archived.is_none() {
        return Err(AppError::Validation(
            "title or archived must be provided".into(),
        ));
    }
    let now = Utc::now().to_rfc3339();
    let archived_at = input.archived.map(|archived| archived.then(|| now.clone()));
    let result = sqlx::query("UPDATE documents SET title = COALESCE(?, title), archived_at = CASE WHEN ? THEN ? ELSE archived_at END, updated_at = ? WHERE id = ? AND user_id = ?")
        .bind(title)
        .bind(input.archived.is_some())
        .bind(archived_at.flatten())
        .bind(&now)
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    get_document(&state, &user_id, &id)
        .await
        .map(|detail| Json(ApiResponse::new(detail)))
}

#[utoipa::path(delete, path = "/api/v1/documents/{id}", params(("id" = String, Path)), security(("bearer_auth" = [])), responses((status = 204, description = "Document deleted"), (status = 404, description = "Document not found or owned by another user")))]
pub async fn delete(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM documents WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/v1/documents/{id}", params(("id" = String, Path)), security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<DocumentDetail>), (status = 404, description = "Document not found or owned by another user")))]
pub async fn detail(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<DocumentDetail>>, AppError> {
    get_document(&state, &user_id, &id)
        .await
        .map(|detail| Json(ApiResponse::new(detail)))
}

async fn get_document(
    state: &AppState,
    user_id: &str,
    id: &str,
) -> Result<DocumentDetail, AppError> {
    let document = sqlx::query_as::<_, DocumentSummary>("SELECT id, title, source_type, original_filename, byte_size, created_at, updated_at, archived_at FROM documents WHERE id = ? AND user_id = ?")
        .bind(id).bind(user_id).fetch_optional(&state.db).await?.ok_or(AppError::NotFound)?;
    let paragraphs = sqlx::query_as::<_, Paragraph>("SELECT id, position, content FROM document_paragraphs WHERE document_id = ? ORDER BY position")
        .bind(id).fetch_all(&state.db).await?;
    Ok(DocumentDetail {
        document,
        paragraphs,
    })
}

fn split_paragraphs(content: &str) -> Vec<String> {
    content
        .replace("\r\n", "\n")
        .split("\n\n")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::split_paragraphs;
    #[test]
    fn splits_blank_line_delimited_paragraphs() {
        assert_eq!(
            split_paragraphs("one\n\n two\r\n\r\nthree"),
            ["one", "two", "three"]
        );
    }
}
