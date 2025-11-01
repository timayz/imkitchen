//! Web server implementation using Axum

use axum::{routing::get, Router};
use imkitchen::Config;
use std::net::SocketAddr;

/// Start the web server
pub async fn serve(config: &Config, port: u16) -> anyhow::Result<()> {
    let app = create_router();

    // Parse host from config
    let host_parts: Vec<u8> = if config.server.host == "0.0.0.0" {
        vec![0, 0, 0, 0]
    } else {
        config
            .server
            .host
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let host = if host_parts.len() == 4 {
        [host_parts[0], host_parts[1], host_parts[2], host_parts[3]]
    } else {
        [0, 0, 0, 0]
    };

    let addr = SocketAddr::from((host, port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the application router
fn create_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
}

/// Root handler
async fn root() -> &'static str {
    "ImKitchen - Meal Planning Application"
}

/// Health check handler
async fn health() -> &'static str {
    "OK"
}
