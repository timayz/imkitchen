use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use sqlx::Row;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn test_register_with_valid_inputs_creates_user() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=test@example.com&password=password123&password_confirm=password123",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // TwinSpark returns 200 OK with ts-location header (progressive enhancement)
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("ts-location").unwrap(), "/dashboard");

    // Process pending events to project to read model
    test_app.process_events().await;

    // Verify user in database
    let user = sqlx::query("SELECT id, email, tier FROM users WHERE email = 'test@example.com'")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(user.get::<String, _>("email"), "test@example.com");
    assert_eq!(user.get::<String, _>("tier"), "free");
}

#[tokio::test]
async fn test_register_with_duplicate_email_returns_error() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    // First registration
    test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=test@example.com&password=password123&password_confirm=password123",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Process events so the user exists in read model
    test_app.process_events().await;

    // Second registration with same email
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=test@example.com&password=password456&password_confirm=password456",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // TwinSpark returns 200 OK with error in body for form swap
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Email already registered"));
}

#[tokio::test]
async fn test_register_with_short_password_returns_error() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool).await;

    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=test@example.com&password=short&password_confirm=short",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // TwinSpark returns 200 OK with error in body for form swap
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("at least 8 characters"));
}

#[tokio::test]
async fn test_register_with_invalid_email_returns_error() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool).await;

    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=invalid-email&password=password123&password_confirm=password123",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // TwinSpark returns 200 OK with error in body for form swap
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("email"));
}

#[tokio::test]
async fn test_successful_registration_sets_jwt_cookie() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool).await;

    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=test@example.com&password=password123&password_confirm=password123",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();

    assert!(cookie.contains("auth_token="));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("Secure"));
    assert!(cookie.contains("SameSite=Lax"));
}

#[tokio::test]
async fn test_successful_registration_redirects_to_dashboard() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool).await;

    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=test@example.com&password=password123&password_confirm=password123",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // TwinSpark returns 200 OK with ts-location header (progressive enhancement)
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("ts-location").unwrap(), "/dashboard");
}
