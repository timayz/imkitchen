pub mod db;
pub mod handlers;
pub mod middleware;
pub mod shutdown;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{info, warn};

pub use db::{create_database_pool_with_retry, DatabaseConfig};
pub use handlers::health::{ComponentHealth, HealthCheckState, HealthResponse, HealthStatus};
pub use shutdown::{GracefulShutdown, ResourceCleanup};

/// Web server application state
#[derive(Clone)]
pub struct AppState {
    pub health_state: HealthCheckState,
}

/// Create the main application router with database pool
pub fn create_app_with_db(db_pool: Option<sqlx::SqlitePool>) -> Router {
    let health_state = HealthCheckState::new(db_pool);

    Router::new()
        .route("/health", get(handlers::health::health_check_with_deps))
        .nest_service("/static", ServeDir::new("crates/imkitchen-web/static"))
        .layer(TraceLayer::new_for_http())
        .with_state(health_state)
}

/// Create the main application router (basic version for backward compatibility)
pub fn create_app() -> Router {
    create_app_with_db(None)
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

/// Start the web server with graceful shutdown support
pub async fn start_server_with_shutdown(
    host: String,
    port: u16,
    db_pool: Option<sqlx::SqlitePool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app_with_db(db_pool.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!(
        "Starting server with graceful shutdown on {}:{}",
        host, port
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Set up graceful shutdown with 30-second timeout
    let shutdown = GracefulShutdown::new(Duration::from_secs(30));

    // Set up resource cleanup
    let mut resource_cleanup = ResourceCleanup::new();
    if let Some(pool) = db_pool {
        resource_cleanup = resource_cleanup.with_db_pool(pool);
    }

    // Create shutdown signal future
    let shutdown_signal = async move {
        shutdown.wait_for_signal().await;
        info!("Shutdown signal received, initiating graceful shutdown");
    };

    // Start server with graceful shutdown using Axum's built-in support
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal);

    // Handle server result and resource cleanup
    match server.await {
        Ok(_) => {
            info!("Server stopped gracefully");
        }
        Err(e) => {
            warn!("Server error during shutdown: {}", e);
        }
    }

    // Execute resource cleanup after server shutdown
    info!("Executing resource cleanup");
    let cleanup_future = resource_cleanup.cleanup();

    // Apply timeout to cleanup operations
    match tokio::time::timeout(Duration::from_secs(30), cleanup_future).await {
        Ok(_) => {
            info!("Resource cleanup completed successfully");
        }
        Err(_) => {
            warn!("Resource cleanup timed out after 30 seconds");
        }
    }

    Ok(())
}
