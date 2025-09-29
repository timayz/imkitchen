use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
    Router,
};
use serde_json::Value;
use std::fs;
use tower::ServiceExt;

use imkitchen_web::{create_app_routes, AppState};

/// Helper function to create test app
async fn create_test_app() -> Router {
    let app_state = AppState::test_default().await;
    create_app_routes(app_state)
}

#[tokio::test]
async fn test_manifest_json_serves_with_correct_content_type() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/static/manifest.json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get("content-type").unwrap();
    assert_eq!(content_type, "application/json");
}

#[tokio::test]
async fn test_manifest_json_contains_required_fields() {
    // Read the manifest file directly to validate its structure
    let manifest_content = fs::read_to_string("crates/imkitchen-web/static/manifest.json")
        .expect("manifest.json should exist");

    let manifest: Value =
        serde_json::from_str(&manifest_content).expect("manifest.json should be valid JSON");

    // Validate required PWA manifest fields
    assert!(
        manifest["name"].is_string(),
        "manifest should have name field"
    );
    assert!(
        manifest["short_name"].is_string(),
        "manifest should have short_name field"
    );
    assert!(
        manifest["start_url"].is_string(),
        "manifest should have start_url field"
    );
    assert!(
        manifest["display"].is_string(),
        "manifest should have display field"
    );
    assert!(
        manifest["theme_color"].is_string(),
        "manifest should have theme_color field"
    );
    assert!(
        manifest["background_color"].is_string(),
        "manifest should have background_color field"
    );
    assert!(
        manifest["icons"].is_array(),
        "manifest should have icons array"
    );

    // Validate icons array structure
    let icons = manifest["icons"].as_array().unwrap();
    assert!(!icons.is_empty(), "icons array should not be empty");

    for icon in icons {
        assert!(icon["src"].is_string(), "each icon should have src field");
        assert!(
            icon["sizes"].is_string(),
            "each icon should have sizes field"
        );
        assert!(icon["type"].is_string(), "each icon should have type field");
    }
}

#[tokio::test]
async fn test_service_worker_serves_with_correct_content_type() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/static/js/service-worker.js")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("javascript"));
}

#[tokio::test]
async fn test_service_worker_contains_cache_functionality() {
    // Read the service worker file directly to validate its functionality
    let sw_content = fs::read_to_string("crates/imkitchen-web/static/js/service-worker.js")
        .expect("service-worker.js should exist");

    // Validate that service worker contains essential PWA functionality
    assert!(
        sw_content.contains("addEventListener"),
        "service worker should have event listeners"
    );
    assert!(
        sw_content.contains("install"),
        "service worker should handle install event"
    );
    assert!(
        sw_content.contains("fetch"),
        "service worker should handle fetch event"
    );
    assert!(
        sw_content.contains("cache"),
        "service worker should implement caching"
    );
}

#[tokio::test]
async fn test_pwa_installation_prompt_handler() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/pwa/install")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return success response for PWA installation trigger
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_base_template_contains_pwa_meta_tags() {
    // This test will validate that the base template includes PWA meta tags
    // We'll read the actual template file to ensure PWA integration
    let base_template = fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
        .expect("base.html template should exist");

    // Check for essential PWA meta tags
    assert!(
        base_template.contains("name=\"viewport\""),
        "base template should have viewport meta tag"
    );
    assert!(
        base_template.contains("name=\"theme-color\""),
        "base template should have theme-color meta tag"
    );
    assert!(
        base_template.contains("rel=\"manifest\""),
        "base template should link to manifest"
    );
    assert!(
        base_template.contains("rel=\"apple-touch-icon\""),
        "base template should have apple-touch-icon"
    );
}

#[cfg(test)]
mod pwa_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_pwa_offline_indicator_endpoint() {
        let app = create_test_app().await;

        let request = Request::builder()
            .uri("/pwa/offline-status")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_pwa_cache_management_endpoint() {
        let app = create_test_app().await;

        let request = Request::builder()
            .uri("/pwa/cache-status")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
