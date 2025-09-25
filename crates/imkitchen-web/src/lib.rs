use axum::{
    extract::State,
    http::StatusCode,
    middleware::from_fn_with_state,
    response::Json,
    routing::{get, post, Router},
};
use imkitchen_core::AppState;
use imkitchen_shared::{AppConfig, AppError, HealthResponse};
use std::net::SocketAddr;
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{error, info};

pub mod handlers;
pub mod middleware;

pub type SharedState = Arc<RwLock<AppState>>;

pub async fn health_handler(
    State(state): State<SharedState>,
) -> Result<Json<HealthResponse>, (StatusCode, String)> {
    let app_state = state.read().await;
    let health_response = app_state.health_check().await;

    match health_response.status.as_str() {
        "healthy" => Ok(Json(health_response)),
        _ => {
            error!("Health check failed: {:?}", health_response);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Service unhealthy: {}", health_response.status),
            ))
        }
    }
}

/// Create protected routes that require authentication
fn create_protected_router(state: SharedState) -> Router<SharedState> {
    Router::new()
        .route("/profile", get(handlers::get_profile))
        .route("/profile", axum::routing::put(handlers::update_profile))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::csrf_protection,
        ))
        .layer(from_fn_with_state(state.clone(), middleware::session_auth))
}

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/", get(handlers::hello_page))
        .route("/login", get(handlers::login_page))
        .route("/login", post(handlers::login_form_handler))
        .route("/register", get(handlers::register_page))
        .route("/register", post(handlers::register_form_handler))
        .route("/api/csrf-token", get(handlers::get_csrf_token_handler))
        .nest_service("/static", ServeDir::new("crates/imkitchen-web/static"))
        .nest("/api/auth", handlers::create_auth_router())
        .nest("/api/user", create_protected_router(state.clone()))
        .layer(CookieManagerLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                )
                .on_request(tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                ),
        )
        .with_state(state)
}

pub async fn start_server(config: AppConfig) -> Result<(), AppError> {
    info!(
        "Starting server on {}:{}",
        config.server.host, config.server.port
    );

    let mut app_state = AppState::new(config.clone());
    app_state.initialize_database().await?;

    let shared_state = Arc::new(RwLock::new(app_state));
    let app = create_router(shared_state);

    let listener = match tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.server.host, config.server.port
    ))
    .await
    {
        Ok(listener) => {
            info!(
                "Server listening on {}:{}",
                config.server.host, config.server.port
            );
            listener
        }
        Err(e) => {
            error!(
                "Failed to bind to {}:{}: {}",
                config.server.host, config.server.port, e
            );
            return Err(AppError::Server(format!(
                "Failed to bind to address: {}",
                e
            )));
        }
    };

    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        error!("Server error: {}", e);
        return Err(AppError::Server(format!("Server failed: {}", e)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use imkitchen_shared::{DatabaseConfig, LoggingConfig, ServerConfig};
    use serde_json::Value;
    use tower::ServiceExt; // for `app.oneshot()`

    fn create_test_config() -> AppConfig {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Use port 0 for testing
            },
            database: DatabaseConfig {
                url: "sqlite::memory:".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }

    async fn create_test_app() -> Router {
        let config = create_test_config();
        let mut app_state = AppState::new(config);
        app_state.initialize_database().await.unwrap();
        let shared_state = Arc::new(RwLock::new(app_state));
        create_router(shared_state)
    }

    async fn create_test_app_no_db() -> Router {
        let config = create_test_config();
        let app_state = AppState::new(config);
        let shared_state = Arc::new(RwLock::new(app_state));
        create_router(shared_state)
    }

    #[tokio::test]
    async fn test_health_endpoint_healthy() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(health["status"], "healthy");
        assert_eq!(health["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(health["database_status"], "Connected");
        assert!(health["uptime_seconds"].is_number());
    }

    #[tokio::test]
    async fn test_health_endpoint_unhealthy() {
        let app = create_test_app_no_db().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_health_endpoint_structure() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: Value = serde_json::from_slice(&body).unwrap();

        // Verify all required fields are present
        assert!(health.get("status").is_some());
        assert!(health.get("version").is_some());
        assert!(health.get("database_status").is_some());
        assert!(health.get("uptime_seconds").is_some());
    }

    #[tokio::test]
    async fn test_nonexistent_endpoint() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("origin", "http://localhost:3000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        // CORS headers should be present due to CorsLayer::new().allow_origin(Any)
        assert!(response
            .headers()
            .get("access-control-allow-origin")
            .is_some());
    }
}
