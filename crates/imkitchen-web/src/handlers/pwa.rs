use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    Form,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Template)]
#[template(path = "offline/offline.html")]
pub struct OfflineTemplate {
    pub title: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct PWAInstallRequest {
    pub user_agent: Option<String>,
    pub platform: Option<String>,
}

#[derive(Serialize)]
pub struct PWAInstallResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct PWAStatusResponse {
    pub online: bool,
    pub cache_size: usize,
    pub last_updated: String,
    pub sw_version: String,
}

/// Handle PWA installation trigger from frontend
pub async fn pwa_install_handler(
    State(_app_state): State<AppState>,
    Form(request): Form<PWAInstallRequest>,
) -> Result<Json<PWAInstallResponse>, StatusCode> {
    // Log PWA installation attempt for analytics
    let user_agent = request.user_agent.unwrap_or_else(|| "Unknown".to_string());
    let platform = request.platform.unwrap_or_else(|| "Unknown".to_string());

    tracing::info!(
        "PWA install triggered - User Agent: {}, Platform: {}",
        user_agent,
        platform
    );

    // Return success response for TwinSpark handling
    Ok(Json(PWAInstallResponse {
        success: true,
        message: "PWA installation prompt triggered successfully".to_string(),
    }))
}

/// Provide offline status information for service worker
pub async fn pwa_offline_status_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<PWAStatusResponse>, StatusCode> {
    // Return current PWA status
    Ok(Json(PWAStatusResponse {
        online: true,  // This endpoint being hit means we're online
        cache_size: 0, // Placeholder - would be calculated from actual cache
        last_updated: chrono::Utc::now().to_rfc3339(),
        sw_version: "v1".to_string(),
    }))
}

/// Return cache status for service worker management
pub async fn pwa_cache_status_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<PWAStatusResponse>, StatusCode> {
    // Return cache information
    Ok(Json(PWAStatusResponse {
        online: true,
        cache_size: 0, // Would be calculated from actual cache size
        last_updated: chrono::Utc::now().to_rfc3339(),
        sw_version: "v1".to_string(),
    }))
}

/// Serve offline fallback page
pub async fn offline_page_handler(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = OfflineTemplate {
        title: "You're Offline".to_string(),
        message: "It looks like you're not connected to the internet. Don't worry - you can still browse your saved recipes and meal plans!".to_string(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Handle manifest.json requests with proper headers
pub async fn manifest_handler(
) -> Result<([(String, String); 1], Json<serde_json::Value>), StatusCode> {
    // Read manifest.json content
    let manifest_content = include_str!("../../static/manifest.json");

    match serde_json::from_str(manifest_content) {
        Ok(manifest) => {
            let headers = [("Content-Type".to_string(), "application/json".to_string())];
            Ok((headers, Json(manifest)))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Handle service worker requests with proper headers
pub async fn service_worker_handler() -> Result<([(String, String); 2], String), StatusCode> {
    // Read service worker content
    let sw_content = include_str!("../../static/js/service-worker.js");

    let headers = [
        (
            "Content-Type".to_string(),
            "application/javascript".to_string(),
        ),
        ("Service-Worker-Allowed".to_string(), "/".to_string()),
    ];

    Ok((headers, sw_content.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    async fn create_test_app() -> Router {
        let app_state = AppState::test_default().await;
        Router::new()
            .route("/pwa/install", axum::routing::post(pwa_install_handler))
            .route(
                "/pwa/offline-status",
                axum::routing::get(pwa_offline_status_handler),
            )
            .route(
                "/pwa/cache-status",
                axum::routing::get(pwa_cache_status_handler),
            )
            .route("/offline", axum::routing::get(offline_page_handler))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_pwa_install_handler() {
        let app = create_test_app().await;

        let request = Request::builder()
            .uri("/pwa/install")
            .method("POST")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("user_agent=TestAgent&platform=TestPlatform"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_pwa_offline_status() {
        let app = create_test_app().await;

        let request = Request::builder()
            .uri("/pwa/offline-status")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_offline_page() {
        let app = create_test_app().await;

        let request = Request::builder()
            .uri("/offline")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
