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
    pub ai_provider: String,
    pub ai_api_base_url: String,
    pub ai_api_key: Option<String>,
    pub ai_model: String,
    pub ai_timeout_seconds: u64,
    pub ai_max_output_tokens: u32,
    pub ai_thinking: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();
        dotenvy::from_filename("backend/.env").ok();
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
        let ai_provider = env::var("AI_PROVIDER").unwrap_or_else(|_| "local-deterministic".into());
        let ai_provider = normalize_ai_provider(&ai_provider)?;
        let ai_api_key = optional_secret("DEEPSEEK_API_KEY")
            .or_else(|| optional_secret("AI_API_KEY"))
            .filter(|value| !value.eq_ignore_ascii_case("replace-me"));
        if ai_provider == "deepseek" && ai_api_key.is_none() {
            return Err(AppError::Config(
                "DEEPSEEK_API_KEY or AI_API_KEY must be set when AI_PROVIDER=deepseek".into(),
            ));
        }
        let ai_timeout_seconds = parse_or("AI_TIMEOUT_SECONDS", 30)?;
        if ai_timeout_seconds == 0 {
            return Err(AppError::Config(
                "AI_TIMEOUT_SECONDS must be greater than zero".into(),
            ));
        }
        let ai_max_output_tokens = parse_or("AI_MAX_OUTPUT_TOKENS", 1200)?;
        if ai_max_output_tokens == 0 {
            return Err(AppError::Config(
                "AI_MAX_OUTPUT_TOKENS must be greater than zero".into(),
            ));
        }
        let ai_thinking = env::var("AI_THINKING").unwrap_or_else(|_| "disabled".into());
        let ai_thinking = normalize_ai_thinking(&ai_thinking)?;
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
            ai_provider,
            ai_api_base_url: env::var("AI_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".into()),
            ai_api_key,
            ai_model: env::var("AI_MODEL").unwrap_or_else(|_| "deepseek-v4-pro".into()),
            ai_timeout_seconds,
            ai_max_output_tokens,
            ai_thinking,
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

fn optional_secret(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_ai_provider(value: &str) -> Result<String, AppError> {
    let provider = value.trim().to_ascii_lowercase();
    match provider.as_str() {
        "local" | "local-deterministic" => Ok("local-deterministic".into()),
        "deepseek" | "deepseek-v4-pro" => Ok("deepseek".into()),
        _ => Err(AppError::Config(format!(
            "AI_PROVIDER must be local-deterministic or deepseek, got {value}"
        ))),
    }
}

fn normalize_ai_thinking(value: &str) -> Result<String, AppError> {
    let thinking = value.trim().to_ascii_lowercase();
    match thinking.as_str() {
        "enabled" | "disabled" => Ok(thinking),
        _ => Err(AppError::Config(format!(
            "AI_THINKING must be enabled or disabled, got {value}"
        ))),
    }
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
