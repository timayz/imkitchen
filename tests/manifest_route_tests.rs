use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;
use common::{create_test_app, setup_test_db};
use imkitchen::routes::AssetsService;

#[tokio::test]
async fn test_manifest_json_served_with_correct_mime_type() {
    let (pool, executor) = setup_test_db().await;
    let test_app = create_test_app((pool, executor)).await;

    // Merge with static assets service
    let app = test_app.router.fallback_service(AssetsService::new());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/manifest.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "manifest.json should return 200 OK"
    );

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Content-Type header should be present")
        .to_str()
        .unwrap();

    // Accept either application/manifest+json or application/json
    // mime_guess may return application/json for .json files
    assert!(
        content_type.contains("application/manifest+json") || content_type.contains("application/json"),
        "manifest.json should have application/manifest+json or application/json MIME type, got: {}",
        content_type
    );

    // Verify the body contains valid JSON
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    let _manifest: serde_json::Value =
        serde_json::from_str(&body_str).expect("manifest.json should contain valid JSON");
}

#[tokio::test]
async fn test_manifest_json_contains_required_fields() {
    let (pool, executor) = setup_test_db().await;
    let test_app = create_test_app((pool, executor)).await;
    let app = test_app.router.fallback_service(AssetsService::new());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/manifest.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    // Verify required PWA manifest fields
    assert!(manifest.get("name").is_some(), "manifest must have 'name'");
    assert!(
        manifest.get("short_name").is_some(),
        "manifest must have 'short_name'"
    );
    assert!(
        manifest.get("start_url").is_some(),
        "manifest must have 'start_url'"
    );
    assert!(
        manifest.get("display").is_some(),
        "manifest must have 'display'"
    );
    assert!(
        manifest.get("icons").is_some(),
        "manifest must have 'icons'"
    );

    assert_eq!(
        manifest["display"], "standalone",
        "display should be 'standalone'"
    );
}

#[tokio::test]
async fn test_app_icons_accessible() {
    let (pool, executor) = setup_test_db().await;
    let test_app = create_test_app((pool, executor)).await;
    let app = test_app.router.fallback_service(AssetsService::new());

    // Test 192x192 icon
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/icons/icon-192.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "icon-192.png should be accessible"
    );
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap(),
        "image/png",
        "icon should have image/png MIME type"
    );

    // Test 512x512 icon
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/icons/icon-512.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "icon-512.png should be accessible"
    );

    // Test Apple touch icon
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/icons/apple-touch-icon.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "apple-touch-icon.png should be accessible"
    );

    // Test maskable icons
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/icons/icon-192-maskable.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "icon-192-maskable.png should be accessible"
    );
}

#[tokio::test]
async fn test_screenshots_accessible() {
    let (pool, executor) = setup_test_db().await;
    let test_app = create_test_app((pool, executor)).await;
    let app = test_app.router.fallback_service(AssetsService::new());

    // Test mobile dashboard screenshot
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/screenshots/dashboard-mobile.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "dashboard-mobile.png should be accessible"
    );

    // Test recipe detail screenshot
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/screenshots/recipe-detail-mobile.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "recipe-detail-mobile.png should be accessible"
    );

    // Test desktop calendar screenshot
    let response = app
        .oneshot(
            Request::builder()
                .uri("/screenshots/meal-calendar-desktop.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "meal-calendar-desktop.png should be accessible"
    );
}
