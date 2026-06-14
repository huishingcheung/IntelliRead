use axum::{
    extract::{FromRequest, FromRequestParts, Multipart, Query, Request},
    http::{StatusCode, request::Parts},
};
use serde::{Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

use crate::error::AppError;

pub struct ApiJson<T>(pub T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        axum::Json::<T>::from_request(request, state)
            .await
            .map(|axum::Json(value)| Self(value))
            .map_err(|rejection| match rejection.status() {
                StatusCode::UNSUPPORTED_MEDIA_TYPE => {
                    AppError::UnsupportedMediaType("Content-Type must be application/json".into())
                }
                _ => AppError::Validation("invalid JSON request body".into()),
            })
    }
}

pub struct ApiQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ApiQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|Query(value)| Self(value))
            .map_err(|_| AppError::Validation("invalid query parameters".into()))
    }
}

pub struct ApiMultipart(pub Multipart);

impl<S> FromRequest<S> for ApiMultipart
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        Multipart::from_request(request, state)
            .await
            .map(Self)
            .map_err(|rejection| match rejection.status() {
                StatusCode::UNSUPPORTED_MEDIA_TYPE => AppError::UnsupportedMediaType(
                    "Content-Type must be multipart/form-data".into(),
                ),
                StatusCode::PAYLOAD_TOO_LARGE => AppError::PayloadTooLarge,
                _ => AppError::Validation("invalid multipart body".into()),
            })
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub success: bool,
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
}
