use std::{env, net::IpAddr};

use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_seconds: i64,
    pub server_host: IpAddr,
    pub server_port: u16,
    pub max_document_bytes: usize,
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();
        let jwt_secret = required("JWT_SECRET")?;
        if jwt_secret.len() < 32 {
            return Err(AppError::Config(
                "JWT_SECRET must contain at least 32 characters".into(),
            ));
        }
        let jwt_expiration_seconds = parse_or("JWT_EXPIRATION_SECONDS", 86_400)?;
        if jwt_expiration_seconds <= 0 {
            return Err(AppError::Config(
                "JWT_EXPIRATION_SECONDS must be greater than zero".into(),
            ));
        }
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://data/intelliread.db?mode=rwc".into()),
            jwt_secret,
            jwt_expiration_seconds,
            server_host: parse_or("SERVER_HOST", "127.0.0.1".parse().unwrap())?,
            server_port: parse_or("SERVER_PORT", 3000)?,
            max_document_bytes: parse_or("MAX_DOCUMENT_BYTES", 2 * 1024 * 1024)?,
            cors_allowed_origins: parse_origins(
                &env::var("CORS_ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:5173".into()),
            )?,
        })
    }
}

fn parse_origins(value: &str) -> Result<Vec<String>, AppError> {
    let origins: Vec<String> = value
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(str::to_string)
        .collect();
    if origins.is_empty() {
        return Err(AppError::Config(
            "CORS_ALLOWED_ORIGINS must contain at least one origin".into(),
        ));
    }
    for origin in &origins {
        let uri: axum::http::Uri = origin
            .parse()
            .map_err(|error| AppError::Config(format!("invalid CORS origin {origin}: {error}")))?;
        if uri.scheme().is_none() || uri.authority().is_none() || uri.path() != "/" {
            return Err(AppError::Config(format!(
                "CORS origin must contain only scheme and authority: {origin}"
            )));
        }
    }
    Ok(origins)
}

fn required(key: &str) -> Result<String, AppError> {
    env::var(key)
        .map_err(|_| AppError::Config(format!("missing required environment variable {key}")))
}

fn parse_or<T>(key: &str, default: T) -> Result<T, AppError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(value) => value
            .parse()
            .map_err(|error| AppError::Config(format!("invalid {key}: {error}"))),
        Err(_) => Ok(default),
    }
}
