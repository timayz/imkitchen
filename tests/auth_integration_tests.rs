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
    // Story 1.4: Registration now redirects to /onboarding instead of /dashboard
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("ts-location").unwrap(),
        "/onboarding"
    );

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
    // Story 1.4: Registration now redirects to /onboarding instead of /dashboard
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("ts-location").unwrap(),
        "/onboarding"
    );
}

// ============================================================================
// LOGIN TESTS (Story 1.2)
// ============================================================================

#[tokio::test]
async fn test_get_login_renders_form() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool).await;

    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify form elements present (AC: 1)
    assert!(body_str.contains("email"));
    assert!(body_str.contains("password"));
    assert!(body_str.contains("type=\"email\""));
}

#[tokio::test]
async fn test_login_with_valid_credentials_succeeds() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool).await;

    // Pre-create user via registration
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

    test_app.process_events().await;

    // Attempt login with valid credentials (AC: 2, 3)
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=test@example.com&password=password123"))
                .unwrap(),
        )
        .await
        .unwrap();

    // TwinSpark returns 200 OK with ts-location header (AC: 5)
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("ts-location").unwrap(), "/dashboard");

    // Verify JWT cookie set (AC: 3)
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
    assert!(cookie.contains("Max-Age=604800")); // 7 days (AC: 8)
}

#[tokio::test]
async fn test_login_with_invalid_email_returns_generic_error() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool).await;

    // Attempt login with non-existent email (AC: 4)
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=nonexistent@example.com&password=password123",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify generic error message (no user enumeration) (AC: 4)
    assert!(body_str.contains("Invalid credentials"));
    // Ensure error message doesn't leak information about email existence
    assert!(!body_str.contains("not found"));
    assert!(!body_str.contains("doesn't exist"));
    assert!(!body_str.contains("no account"));
}

#[tokio::test]
async fn test_login_with_incorrect_password_returns_generic_error() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool).await;

    // Pre-create user
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

    test_app.process_events().await;

    // Attempt login with incorrect password (AC: 4)
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=test@example.com&password=wrongpassword"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify generic error message (same as invalid email) (AC: 4)
    assert!(body_str.contains("Invalid credentials"));
    // Ensure error message doesn't leak information about password being wrong
    assert!(!body_str.contains("incorrect"));
    assert!(!body_str.contains("wrong"));
    assert!(!body_str.contains("mismatch"));
}

#[tokio::test]
async fn test_login_jwt_includes_correct_claims() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool).await;

    // Pre-create user
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

    test_app.process_events().await;

    // Login
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=test@example.com&password=password123"))
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

    // Extract token from cookie
    let token = cookie
        .split(';')
        .next()
        .unwrap()
        .trim_start_matches("auth_token=");

    // Decode and verify claims (AC: 7)
    let claims = user::validate_jwt(token, "test_secret_key_minimum_32_characters_long").unwrap();

    assert!(!claims.sub.is_empty()); // user_id
    assert_eq!(claims.email, "test@example.com");
    assert_eq!(claims.tier, "free");
    assert!(claims.exp > claims.iat); // Expiration is in future
}
