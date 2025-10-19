// Integration tests for Service Worker functionality - Story 5.2
// Tests service worker registration, offline route, and asset serving

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

/// Test that /offline route returns 200 OK
#[tokio::test]
async fn test_offline_route_returns_ok() {
    let (pool, executor) = common::setup_test_db().await;

    // Create app with offline route
    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    // Request /offline
    let response = app
        .oneshot(
            Request::builder()
                .uri("/offline")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify response contains offline page content
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_str.contains("You're Offline"));
    assert!(body_str.contains("cached recipes"));
}

/// Test that /offline route returns HTML content type
#[tokio::test]
async fn test_offline_route_content_type() {
    let (pool, executor) = common::setup_test_db().await;

    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/offline")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check Content-Type header
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();

    assert!(content_type.contains("text/html"));
}

/// Test that service worker is served from /static/sw.js
#[tokio::test]
async fn test_service_worker_served() {
    let (pool, executor) = common::setup_test_db().await;

    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/static/sw.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check Content-Type header is application/javascript
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();

    assert!(
        content_type.contains("application/javascript") || content_type.contains("text/javascript")
    );
}

/// Test that sw-register.js is served
#[tokio::test]
async fn test_sw_register_script_served() {
    let (pool, executor) = common::setup_test_db().await;

    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/static/js/sw-register.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify script contains registration logic
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_str.contains("serviceWorker"));
    assert!(body_str.contains("register"));
}

/// Test that offline-indicator.js is served
#[tokio::test]
async fn test_offline_indicator_script_served() {
    let (pool, executor) = common::setup_test_db().await;

    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/static/js/offline-indicator.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify script contains offline/online event listeners
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_str.contains("offline"));
    assert!(body_str.contains("online"));
}

/// Test that non-existent static files return 404
#[tokio::test]
async fn test_nonexistent_static_file_returns_404() {
    let (pool, executor) = common::setup_test_db().await;

    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/static/js/nonexistent-file.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test that offline page is accessible without authentication
#[tokio::test]
async fn test_offline_page_no_auth_required() {
    let (pool, executor) = common::setup_test_db().await;

    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    // Request without auth cookie
    let response = app
        .oneshot(
            Request::builder()
                .uri("/offline")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should succeed without authentication
    assert_eq!(response.status(), StatusCode::OK);
}

/// Test that offline page contains helpful messaging
#[tokio::test]
async fn test_offline_page_contains_helpful_content() {
    let (pool, executor) = common::setup_test_db().await;

    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/offline")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify helpful messaging is present
    assert!(body_str.contains("You're Offline"));
    assert!(body_str.contains("No internet connection"));
    assert!(body_str.contains("cached recipes"));
    assert!(body_str.contains("Try Again"));
}
