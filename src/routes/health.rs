use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, NoContent},
};
use sqlx::SqlitePool;

/// GET /health - Liveness probe
/// Returns 200 OK if the process is alive
/// Used by Kubernetes liveness probe
pub async fn health() -> NoContent {
    NoContent
}

/// GET /ready - Readiness probe
/// Returns 200 OK if the application is ready to serve traffic
/// Checks:
/// - Database connection is alive
/// - evento is initialized (implicitly via database check)
pub async fn ready(State(pool): State<SqlitePool>) -> impl IntoResponse {
    // Check database connectivity
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => NoContent.into_response(),
        Err(e) => {
            tracing::error!("Readiness check failed: database unavailable - {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, "database unavailable").into_response()
        }
    }
}
