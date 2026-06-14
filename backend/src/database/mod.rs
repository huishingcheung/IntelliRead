use std::path::Path;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::error::AppError;

pub async fn connect(database_url: &str) -> Result<SqlitePool, AppError> {
    ensure_parent_directory(database_url)?;
    let options: SqliteConnectOptions = database_url
        .parse()
        .map_err(|error| AppError::Config(format!("invalid DATABASE_URL: {error}")))?;
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options.foreign_keys(true))
        .await?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|error| AppError::Config(format!("migration failed: {error}")))?;
    Ok(pool)
}

fn ensure_parent_directory(database_url: &str) -> Result<(), AppError> {
    let Some(path) = database_url
        .strip_prefix("sqlite://")
        .and_then(|value| value.split('?').next())
    else {
        return Ok(());
    };
    if path == ":memory:" {
        return Ok(());
    }
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            AppError::Config(format!("cannot create database directory: {error}"))
        })?;
    }
    Ok(())
}
