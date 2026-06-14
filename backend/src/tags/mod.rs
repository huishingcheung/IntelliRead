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
pub struct Tag {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TagRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetDocumentTagsRequest {
    pub tag_ids: Vec<String>,
}

#[utoipa::path(post, path = "/api/v1/tags", request_body = TagRequest, security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Tag>), (status = 409, description = "Tag name already exists")))]
pub async fn create(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    ApiJson(input): ApiJson<TagRequest>,
) -> Result<Json<ApiResponse<Tag>>, AppError> {
    let name = validate_name(&input.name)?;
    let tag = Tag {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    let result =
        sqlx::query("INSERT INTO tags (id, user_id, name, created_at) VALUES (?, ?, ?, ?)")
            .bind(&tag.id)
            .bind(&user_id)
            .bind(&tag.name)
            .bind(&tag.created_at)
            .execute(&state.db)
            .await;
    match result {
        Ok(_) => Ok(Json(ApiResponse::new(tag))),
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
            Err(AppError::Conflict("tag name already exists".into()))
        }
        Err(error) => Err(error.into()),
    }
}

#[utoipa::path(get, path = "/api/v1/tags", security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Vec<Tag>>)))]
pub async fn list(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Tag>>>, AppError> {
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT id, name, created_at FROM tags WHERE user_id = ? ORDER BY name",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(ApiResponse::new(tags)))
}

#[utoipa::path(put, path = "/api/v1/tags/{id}", params(("id" = String, Path)), request_body = TagRequest, security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Tag>), (status = 404, description = "Tag not found")))]
pub async fn update(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    ApiJson(input): ApiJson<TagRequest>,
) -> Result<Json<ApiResponse<Tag>>, AppError> {
    let name = validate_name(&input.name)?;
    let result = sqlx::query("UPDATE tags SET name = ? WHERE id = ? AND user_id = ?")
        .bind(name)
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await;
    match result {
        Ok(result) if result.rows_affected() == 0 => return Err(AppError::NotFound),
        Ok(_) => {}
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
            return Err(AppError::Conflict("tag name already exists".into()));
        }
        Err(error) => return Err(error.into()),
    }
    let tag = sqlx::query_as::<_, Tag>(
        "SELECT id, name, created_at FROM tags WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(ApiResponse::new(tag)))
}

#[utoipa::path(delete, path = "/api/v1/tags/{id}", params(("id" = String, Path)), security(("bearer_auth" = [])), responses((status = 204, description = "Tag deleted")))]
pub async fn delete(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM tags WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(put, path = "/api/v1/documents/{id}/tags", params(("id" = String, Path)), request_body = SetDocumentTagsRequest, security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Vec<Tag>>), (status = 404, description = "Document or tag not found")))]
pub async fn set_document_tags(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
    ApiJson(input): ApiJson<SetDocumentTagsRequest>,
) -> Result<Json<ApiResponse<Vec<Tag>>>, AppError> {
    ensure_document(&state, &user_id, &document_id).await?;
    let mut tag_ids = input.tag_ids;
    tag_ids.sort();
    tag_ids.dedup();
    for tag_id in &tag_ids {
        let exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE id = ? AND user_id = ?")
                .bind(tag_id)
                .bind(&user_id)
                .fetch_one(&state.db)
                .await?;
        if exists == 0 {
            return Err(AppError::NotFound);
        }
    }
    let mut tx = state.db.begin().await?;
    sqlx::query("DELETE FROM document_tags WHERE document_id = ?")
        .bind(&document_id)
        .execute(&mut *tx)
        .await?;
    let now = Utc::now().to_rfc3339();
    for tag_id in tag_ids {
        sqlx::query("INSERT INTO document_tags (document_id, tag_id, created_at) VALUES (?, ?, ?)")
            .bind(&document_id)
            .bind(tag_id)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    document_tags(&state, &user_id, &document_id).await
}

#[utoipa::path(get, path = "/api/v1/documents/{id}/tags", params(("id" = String, Path)), security(("bearer_auth" = [])), responses((status = 200, body = ApiResponse<Vec<Tag>>)))]
pub async fn get_document_tags(
    AuthUser { id: user_id }: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Tag>>>, AppError> {
    ensure_document(&state, &user_id, &document_id).await?;
    document_tags(&state, &user_id, &document_id).await
}

async fn document_tags(
    state: &AppState,
    user_id: &str,
    document_id: &str,
) -> Result<Json<ApiResponse<Vec<Tag>>>, AppError> {
    let tags = sqlx::query_as::<_, Tag>("SELECT t.id, t.name, t.created_at FROM tags t JOIN document_tags dt ON dt.tag_id = t.id WHERE dt.document_id = ? AND t.user_id = ? ORDER BY t.name")
        .bind(document_id)
        .bind(user_id)
        .fetch_all(&state.db)
        .await?;
    Ok(Json(ApiResponse::new(tags)))
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

fn validate_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 50 {
        return Err(AppError::Validation(
            "tag name must contain 1 to 50 characters".into(),
        ));
    }
    Ok(name)
}
