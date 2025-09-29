pub mod db;
pub mod handlers;
pub mod metrics;
pub mod middleware;
pub mod shutdown;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{info, warn};

pub use db::{create_database_pool_with_retry, DatabaseConfig};
pub use handlers::health::{ComponentHealth, HealthCheckState, HealthResponse, HealthStatus};
pub use handlers::metrics::metrics_handler;
pub use metrics::AppMetrics;
pub use shutdown::{GracefulShutdown, ResourceCleanup};

// Import user authentication services
use imkitchen_user::commands::register_user::RegisterUserService;
use imkitchen_user::commands::ProfileCommandHandler;
use imkitchen_user::queries::UserQueryHandler;
use imkitchen_user::services::login_service::DirectLoginService;

/// Enhanced web server application state with metrics and user services
#[derive(Clone)]
pub struct AppState {
    pub health_state: HealthCheckState,
    pub metrics: AppMetrics,
    pub login_service: Option<DirectLoginService>,
    pub user_query_handler: Option<UserQueryHandler>,
    pub register_service: Option<RegisterUserService>,
    pub profile_handler: Option<ProfileCommandHandler>,
}

/// Create the main application router with database pool and metrics
pub fn create_app_with_metrics(db_pool: Option<sqlx::SqlitePool>, metrics: AppMetrics) -> Router {
    let health_state = HealthCheckState::new(db_pool.clone());

    // Initialize user services if database is available
    let (login_service, user_query_handler, register_service, profile_handler) =
        if let Some(ref pool) = db_pool {
            let login_service = DirectLoginService::new(pool.clone());
            let user_query_handler = UserQueryHandler::new(pool.clone());
            let register_service = RegisterUserService::with_database(pool.clone());
            let profile_handler = ProfileCommandHandler::new(pool.clone());
            (
                Some(login_service),
                Some(user_query_handler),
                Some(register_service),
                Some(profile_handler),
            )
        } else {
            (None, None, None, None)
        };

    // Create unified app state
    let app_state = AppState {
        health_state: health_state.clone(),
        metrics: metrics.clone(),
        login_service,
        user_query_handler,
        register_service,
        profile_handler,
    };

    // Set application info in metrics
    metrics.set_app_info(env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_RUST_VERSION"));

    // Create health router with health state
    let health_router = Router::new()
        .route("/health", get(handlers::health::health_check_with_deps))
        .with_state(health_state);

    // Create metrics router with metrics state
    let metrics_router = Router::new()
        .route("/metrics", get(handlers::metrics::metrics_handler))
        .with_state(metrics.clone());

    // Create auth router with app state
    let mut auth_router = Router::new()
        .route("/auth/login", get(handlers::auth::login_page))
        .route("/auth/login", post(handlers::auth::login_handler))
        .route("/auth/register", get(handlers::auth::register_page))
        .route("/auth/register", post(handlers::auth::register_handler))
        .with_state(app_state.clone());

    // Create dashboard router with auth middleware
    let dashboard_router = Router::new()
        .route("/dashboard", get(handlers::dashboard::user_dashboard))
        .route("/", get(handlers::dashboard::user_dashboard)) // Root redirects to dashboard for demo
        .layer(from_fn_with_state(
            app_state.clone(),
            crate::middleware::auth::auth_middleware,
        ))
        .with_state(app_state.clone());

    // Create profile router with auth middleware
    let profile_router = Router::new()
        .route("/profile", get(handlers::profile::profile_page))
        .route("/profile/edit", get(handlers::profile::profile_edit_page))
        .route(
            "/profile/update",
            post(handlers::profile::update_profile_handler),
        )
        .route(
            "/profile/dietary",
            post(handlers::profile::update_dietary_restrictions_handler),
        )
        .route(
            "/profile/validate",
            post(handlers::profile::validate_profile_handler),
        )
        .layer(from_fn_with_state(
            app_state.clone(),
            crate::middleware::auth::auth_middleware,
        ))
        .with_state(app_state.clone());

    // Add async validation routes if we have a database pool
    if let Some(pool) = db_pool.clone() {
        let async_validation_router = Router::new()
            .route(
                "/api/validate/email",
                get(handlers::async_validation::validate_email_async),
            )
            .route(
                "/api/validate/email",
                post(handlers::async_validation::validate_email_form),
            )
            .route(
                "/api/validate/username",
                get(handlers::async_validation::validate_username_async),
            )
            .with_state(pool);

        auth_router = auth_router.merge(async_validation_router);
    }

    // Combine routers with middleware layers
    Router::new()
        .merge(dashboard_router)
        .merge(profile_router)
        .merge(health_router)
        .merge(metrics_router)
        .merge(auth_router)
        .nest_service("/static", ServeDir::new("crates/imkitchen-web/static"))
        .layer(from_fn_with_state(
            metrics,
            crate::middleware::metrics_middleware,
        ))
        .layer(TraceLayer::new_for_http())
}

/// Create the main application router with database pool
pub fn create_app_with_db(db_pool: Option<sqlx::SqlitePool>) -> Router {
    let metrics = AppMetrics::new().expect("Failed to create metrics");
    create_app_with_metrics(db_pool, metrics)
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
    let metrics = AppMetrics::new().expect("Failed to create metrics");
    let app = create_app_with_metrics(db_pool.clone(), metrics.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!(
        "Starting server with graceful shutdown on {}:{}",
        host, port
    );

    // Start database metrics collection task if we have a pool
    if let Some(ref pool) = db_pool {
        let pool_clone = pool.clone();
        let metrics_clone = metrics.clone();
        tokio::spawn(async move {
            update_database_metrics_periodically(pool_clone, metrics_clone).await;
        });
    }

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

/// Periodically update database connection metrics
async fn update_database_metrics_periodically(db_pool: sqlx::SqlitePool, metrics: AppMetrics) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        let stats = db::get_pool_stats(&db_pool);
        metrics.update_db_connections(stats.size, stats.idle);

        // Update uptime metrics
        let uptime = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        metrics.update_uptime(uptime);
    }
}
