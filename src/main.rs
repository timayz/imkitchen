use axum::Router;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod config;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;

use config::{Settings, Environment};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "imkitchen=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let settings = Settings::new().expect("Failed to load configuration");
    
    tracing::info!(
        "Starting ImKitchen in {} mode on {}:{}",
        match settings.app.environment {
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
        },
        settings.server.host,
        settings.server.port
    );

    // Build our application with routes
    let app = Router::new()
        .merge(routes::create_routes())
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

    // Create server address
    let listener = tokio::net::TcpListener::bind(&format!("{}:{}", settings.server.host, settings.server.port))
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server running on http://{}:{}", settings.server.host, settings.server.port);

    // Start the server
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}