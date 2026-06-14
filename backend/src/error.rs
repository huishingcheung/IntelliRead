use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::response::{ErrorBody, ErrorDetail};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("authentication required")]
    Unauthorized,
    #[error("access denied")]
    Forbidden,
    #[error("resource not found")]
    NotFound,
    #[error("resource already exists: {0}")]
    Conflict(String),
    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),
    #[error("payload exceeds the configured limit")]
    PayloadTooLarge,
    #[error("database operation failed")]
    Database(#[from] sqlx::Error),
    #[error("internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, public_message) = match &self {
            Self::Config(_) | Self::Database(_) | Self::Internal => {
                tracing::error!(error = %self, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "internal server error".to_string(),
                )
            }
            Self::Validation(message) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message.clone())
            }
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string()),
            Self::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", self.to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            Self::Conflict(message) => (StatusCode::CONFLICT, "CONFLICT", message.clone()),
            Self::UnsupportedMediaType(message) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                message.clone(),
            ),
            Self::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "PAYLOAD_TOO_LARGE",
                self.to_string(),
            ),
        };
        (
            status,
            Json(ErrorBody {
                success: false,
                error: ErrorDetail {
                    code,
                    message: public_message,
                },
            }),
        )
            .into_response()
    }
}
