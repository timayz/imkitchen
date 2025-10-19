use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use serde_json::json;
use sqlx::SqlitePool;

/// GET /offline - Offline fallback page for service worker
/// Returns a cached HTML page when the user is offline
pub async fn offline() -> impl IntoResponse {
    let html = include_str!("../../templates/offline.html");
    Html(html)
}

/// GET /health - Liveness probe
/// Returns 200 OK if the process is alive
/// Used by Kubernetes liveness probe
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// GET /ready - Readiness probe
/// Returns 200 OK if the application is ready to serve traffic
/// Checks:
/// - Database connection is alive
/// - evento is initialized (implicitly via database check)
pub async fn ready(State(pool): State<SqlitePool>) -> impl IntoResponse {
    // Check database connectivity
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "ready"}))),
        Err(e) => {
            tracing::error!("Readiness check failed: database unavailable - {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "not_ready",
                    "reason": "database_unavailable"
                })),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_health_endpoint() {
        let response = health().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_endpoint_with_valid_db() {
        // Create in-memory database
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let response = ready(State(pool)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
