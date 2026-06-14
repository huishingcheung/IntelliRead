use std::sync::Arc;

use intelliread_backend::{app, config::Config, database, state::AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = Config::from_env()?;
    let address = (config.server_host, config.server_port);
    let db = database::connect(&config.database_url).await?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(?address, "IntelliRead backend listening");
    axum::serve(listener, app(Arc::new(AppState { db, config }))).await?;
    Ok(())
}
