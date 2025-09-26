pub mod handlers;
pub mod middleware;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

/// Web server application state
#[derive(Clone)]
pub struct AppState {
    // Application state will be added as we implement features
}

/// Create the main application router
pub fn create_app() -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .nest_service("/static", ServeDir::new("crates/imkitchen-web/static"))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {})
}

/// Start the web server
pub async fn start_server(host: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Starting server on {}:{}", host, port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
