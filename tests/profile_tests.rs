use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

mod common;

/// Test GET /profile requires authentication
#[tokio::test]
async fn test_get_profile_requires_auth() {
    let (pool, _executor) = common::setup_test_db().await;
    let app = common::create_test_app((pool.clone(), _executor)).await;

    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized or 303 redirect to login
    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::SEE_OTHER
    );
}

/// Test POST /profile requires authentication
#[tokio::test]
async fn test_post_profile_requires_auth() {
    let (pool, _executor) = common::setup_test_db().await;
    let app = common::create_test_app((pool.clone(), _executor)).await;

    let form_data = "dietary_restrictions=vegetarian&household_size=2&availability_start=18:00&availability_duration=45";

    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/profile")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized or 303 redirect to login
    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::SEE_OTHER
    );
}
