use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Json,
    extract::{FromRef, FromRequestParts, State},
    http::{StatusCode, header, request::Parts},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    response::{ApiJson, ApiResponse},
    state::AppState,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 128))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, FromRow)]
struct UserRow {
    id: String,
    username: String,
    email: String,
    password_hash: String,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: String,
}

#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterRequest, responses((status = 201, body = ApiResponse<UserResponse>), (status = 400, body = crate::response::ErrorBody), (status = 409, body = crate::response::ErrorBody), (status = 415, body = crate::response::ErrorBody)))]
pub async fn register(
    State(state): State<Arc<AppState>>,
    ApiJson(input): ApiJson<RegisterRequest>,
) -> Result<(StatusCode, Json<ApiResponse<UserResponse>>), AppError> {
    input
        .validate()
        .map_err(|error| AppError::Validation(error.to_string()))?;
    let exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = ? OR email = ?")
            .bind(input.username.trim())
            .bind(input.email.trim())
            .fetch_one(&state.db)
            .await?;
    if exists > 0 {
        return Err(AppError::Conflict(
            "username or email already exists".into(),
        ));
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let hash = hash_password(&input.password)?;
    let result = sqlx::query("INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(input.username.trim()).bind(input.email.trim().to_lowercase()).bind(hash).bind(&now).bind(&now).execute(&state.db).await?;
    if result.rows_affected() != 1 {
        return Err(AppError::Internal);
    }
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::new(UserResponse {
            id,
            username: input.username.trim().into(),
            email: input.email.trim().to_lowercase(),
            created_at: now,
        })),
    ))
}

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest, responses((status = 200, body = ApiResponse<LoginResponse>), (status = 400, body = crate::response::ErrorBody), (status = 401, body = crate::response::ErrorBody), (status = 415, body = crate::response::ErrorBody)))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    ApiJson(input): ApiJson<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    input
        .validate()
        .map_err(|error| AppError::Validation(error.to_string()))?;
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE email = ?",
    )
    .bind(input.email.trim().to_lowercase())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;
    verify_password(&input.password, &user.password_hash)?;
    let token = issue_token(&user.id, &state)?;
    Ok(Json(ApiResponse::new(LoginResponse {
        access_token: token,
        token_type: "Bearer",
        expires_in: state.config.jwt_expiration_seconds,
        user: UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        },
    })))
}

impl<S> FromRequestParts<S> for AuthUser
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = Arc::<AppState>::from_ref(state);
        let value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        let token = value
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?
        .claims;
        Ok(Self { id: claims.sub })
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|hash| hash.to_string())
        .map_err(|_| AppError::Internal)
}

fn verify_password(password: &str, encoded: &str) -> Result<(), AppError> {
    let hash = PasswordHash::new(encoded).map_err(|_| AppError::Internal)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .map_err(|_| AppError::Unauthorized)
}

fn issue_token(user_id: &str, state: &AppState) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.into(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::seconds(state.config.jwt_expiration_seconds)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal)
}
