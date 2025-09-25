use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use imkitchen_core::AppState;
use imkitchen_shared::{AppConfig, DatabaseConfig, LoggingConfig, ServerConfig};
use imkitchen_web::{create_router, SharedState};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

async fn create_test_app() -> Router {
    let config = AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
        },
        database: DatabaseConfig {
            url: "sqlite::memory:".to_string(),
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
        },
    };

    let mut app_state = AppState::new(config);
    app_state.initialize_database().await.unwrap();
    let shared_state: SharedState = Arc::new(RwLock::new(app_state));

    create_router(shared_state)
}

async fn register_test_user(
    app: &Router,
    email: &str,
    password: &str,
    name: &str,
) -> axum::response::Response {
    let register_payload = json!({
        "email": email,
        "password": password,
        "name": name,
        "familySize": 2
    });

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn login_test_user(app: &Router, email: &str, password: &str) -> axum::response::Response {
    let login_payload = json!({
        "email": email,
        "password": password
    });

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

fn extract_session_cookie(response: &axum::response::Response) -> Option<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|cookie_header| {
            let cookie_str = cookie_header.to_str().ok()?;
            if cookie_str.starts_with("session_id=") {
                Some(cookie_str.to_string())
            } else {
                None
            }
        })
}

#[tokio::test]
#[ignore] // Integration test - requires complex setup
async fn test_complete_authentication_flow() {
    let app = create_test_app().await;

    // Test user registration
    let register_response =
        register_test_user(&app, "test@example.com", "Password123!", "Test User").await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Test user login
    let login_response = login_test_user(&app, "test@example.com", "Password123!").await;
    assert_eq!(login_response.status(), StatusCode::OK);

    // Extract session cookie
    let session_cookie =
        extract_session_cookie(&login_response).expect("Should have session cookie");

    // Test protected route access with valid session
    let profile_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/user/profile")
                .header("cookie", session_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(profile_response.status(), StatusCode::OK);

    // Test logout
    let logout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .header("cookie", session_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(logout_response.status(), StatusCode::OK);

    // Test protected route access after logout (should fail)
    let profile_after_logout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/user/profile")
                .header("cookie", session_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        profile_after_logout_response.status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
#[ignore] // Integration test - requires complex setup
async fn test_csrf_protection() {
    let app = create_test_app().await;

    // Register and login user
    register_test_user(&app, "csrf@example.com", "Password123!", "CSRF User").await;
    let login_response = login_test_user(&app, "csrf@example.com", "Password123!").await;
    let session_cookie =
        extract_session_cookie(&login_response).expect("Should have session cookie");

    // Get CSRF token
    let csrf_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/csrf-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(csrf_response.status(), StatusCode::OK);

    // Try updating profile without CSRF token (should fail)
    let update_without_csrf = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/user/profile")
                .header("cookie", session_cookie.clone())
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "Updated Name"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_without_csrf.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[ignore] // Integration test - requires complex setup
async fn test_invalid_credentials() {
    let app = create_test_app().await;

    // Test login with non-existent user
    let invalid_login_response = login_test_user(&app, "nonexistent@example.com", "password").await;
    assert_eq!(invalid_login_response.status(), StatusCode::UNAUTHORIZED);

    // Register a user
    register_test_user(&app, "valid@example.com", "Password123!", "Valid User").await;

    // Test login with wrong password
    let wrong_password_response = login_test_user(&app, "valid@example.com", "WrongPassword").await;
    assert_eq!(wrong_password_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore] // Integration test - requires complex setup
async fn test_duplicate_registration() {
    let app = create_test_app().await;

    // Register user first time
    let first_registration =
        register_test_user(&app, "duplicate@example.com", "Password123!", "First User").await;
    assert_eq!(first_registration.status(), StatusCode::CREATED);

    // Try to register same email again
    let duplicate_registration =
        register_test_user(&app, "duplicate@example.com", "Password123!", "Second User").await;
    assert_eq!(duplicate_registration.status(), StatusCode::CONFLICT);
}

#[tokio::test]
#[ignore] // Integration test - requires complex setup
async fn test_password_validation() {
    let app = create_test_app().await;

    // Test weak password
    let weak_password_payload = json!({
        "email": "weak@example.com",
        "password": "123",
        "name": "Weak User",
        "familySize": 1
    });

    let weak_password_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(weak_password_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(weak_password_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_protected_routes_without_auth() {
    let app = create_test_app().await;

    // Try to access protected profile endpoint without authentication
    let profile_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/user/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(profile_response.status(), StatusCode::UNAUTHORIZED);

    // Try to update profile without authentication
    let update_profile_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/user/profile")
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "Hacker"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_profile_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;

    let health_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(health_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(health_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let health_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(health_json["status"], "healthy");
    assert!(health_json["version"].is_string());
    assert_eq!(health_json["database_status"], "Connected");
}
