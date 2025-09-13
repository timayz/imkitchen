use axum::{
    http::StatusCode,
    Router,
};
use axum_test::TestServer;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use imkitchen::{
    models::user::{AuthResponse, UserProfile},
    repositories::UserRepository,
    services::AuthService,
};

async fn _setup_test_app() -> (TestServer, PgPool, redis::Client) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/imkitchen_test".to_string());
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let redis_client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client");
    
    let user_repo = UserRepository::new(pool.clone());
    let jwt_secret = "test_jwt_secret_32_characters_long".to_string();
    let auth_service = AuthService::new(user_repo, redis_client.clone(), jwt_secret);
    
    // Create a simple router for testing
    let app = Router::new()
        .nest("/auth", imkitchen::routes::pages::create_auth_routes())
        .with_state(auth_service);
    
    let server = TestServer::new(app).unwrap();
    
    (server, pool, redis_client)
}

async fn cleanup_test_user(pool: &PgPool, email: &str) {
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await;
}

#[ignore]
#[tokio::test]
async fn test_api_register_success() {
    let (server, pool, _redis) = _setup_test_app().await;
    let test_email = format!("test_api_register_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    let request_body = json!({
        "email": test_email,
        "password": "password123",
        "skill_level": "intermediate",
        "household_size": 2,
        "dietary_preferences": ["vegetarian"]
    });
    
    let response = server
        .post("/auth/api/register")
        .json(&request_body)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let user: UserProfile = response.json();
    assert_eq!(user.email, test_email);
    assert_eq!(user.skill_level, "intermediate");
    assert_eq!(user.household_size, 2);
    assert!(user.dietary_preferences.contains(&"vegetarian".to_string()));
    assert!(!user.email_verified);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[ignore]
#[tokio::test]
async fn test_api_register_duplicate_email() {
    let (server, pool, _redis) = _setup_test_app().await;
    let test_email = format!("test_api_duplicate_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    let request_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    // First registration should succeed
    let response1 = server
        .post("/auth/api/register")
        .json(&request_body)
        .await;
    
    assert_eq!(response1.status_code(), StatusCode::OK);
    
    // Second registration should fail with conflict
    let response2 = server
        .post("/auth/api/register")
        .json(&request_body)
        .await;
    
    assert_eq!(response2.status_code(), StatusCode::CONFLICT);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[ignore]
#[tokio::test]
async fn test_api_register_invalid_input() {
    let (server, _pool, _redis) = _setup_test_app().await;
    
    // Test with missing email
    let request_body = json!({
        "password": "password123"
    });
    
    let response = server
        .post("/auth/api/register")
        .json(&request_body)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    
    // Test with short password
    let request_body = json!({
        "email": "test@example.com",
        "password": "short"
    });
    
    let response = server
        .post("/auth/api/register")
        .json(&request_body)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[ignore]
#[tokio::test]
async fn test_api_login_success() {
    let (server, pool, _redis) = _setup_test_app().await;
    let test_email = format!("test_api_login_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Register a user first
    let register_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    let register_response = server
        .post("/auth/api/register")
        .json(&register_body)
        .await;
    
    assert_eq!(register_response.status_code(), StatusCode::OK);
    let user: UserProfile = register_response.json();
    
    // Manually verify the user for testing
    sqlx::query("UPDATE users SET email_verified = TRUE WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .unwrap();
    
    // Test login
    let login_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    let response = server
        .post("/auth/api/login")
        .json(&login_body)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let auth_response: AuthResponse = response.json();
    assert!(!auth_response.access_token.is_empty());
    assert_eq!(auth_response.user.email, test_email);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[ignore]
#[tokio::test]
async fn test_api_login_invalid_credentials() {
    let (server, pool, _redis) = _setup_test_app().await;
    let test_email = format!("test_api_invalid_{}@example.com", Uuid::new_v4());
    
    // Test with non-existent user
    let login_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    let response = server
        .post("/auth/api/login")
        .json(&login_body)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    
    // Register a user
    let register_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    server
        .post("/auth/api/register")
        .json(&register_body)
        .await;
    
    // Test with wrong password
    let login_body = json!({
        "email": test_email,
        "password": "wrongpassword"
    });
    
    let response = server
        .post("/auth/api/login")
        .json(&login_body)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[ignore]
#[tokio::test]
async fn test_api_login_email_not_verified() {
    let (server, pool, _redis) = _setup_test_app().await;
    let test_email = format!("test_api_not_verified_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Register a user (email not verified by default)
    let register_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    server
        .post("/auth/api/register")
        .json(&register_body)
        .await;
    
    // Test login with unverified email
    let login_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    let response = server
        .post("/auth/api/login")
        .json(&login_body)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[ignore]
#[tokio::test]
async fn test_protected_endpoint_without_token() {
    let (server, _pool, _redis) = _setup_test_app().await;
    
    // Try to access profile without authentication
    let response = server
        .get("/auth/api/profile")
        .await;
    
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[ignore]
#[tokio::test]
async fn test_protected_endpoint_with_valid_token() {
    let (server, pool, _redis) = _setup_test_app().await;
    let test_email = format!("test_api_protected_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Register and verify a user
    let register_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    let register_response = server
        .post("/auth/api/register")
        .json(&register_body)
        .await;
    
    let user: UserProfile = register_response.json();
    
    // Manually verify the user
    sqlx::query("UPDATE users SET email_verified = TRUE WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .unwrap();
    
    // Login to get token
    let login_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    let login_response = server
        .post("/auth/api/login")
        .json(&login_body)
        .await;
    
    let auth_response: AuthResponse = login_response.json();
    
    // Access protected endpoint with token
    let response = server
        .get("/auth/api/profile")
        .authorization_bearer(&auth_response.access_token)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[ignore]
#[tokio::test]
async fn test_protected_endpoint_with_invalid_token() {
    let (server, _pool, _redis) = _setup_test_app().await;
    
    // Try to access profile with invalid token
    let response = server
        .get("/auth/api/profile")
        .authorization_bearer("invalid_token")
        .await;
    
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[ignore]
#[tokio::test]
async fn test_logout() {
    let (server, pool, _redis) = _setup_test_app().await;
    let test_email = format!("test_api_logout_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Register, verify, and login
    let register_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    let register_response = server
        .post("/auth/api/register")
        .json(&register_body)
        .await;
    
    let user: UserProfile = register_response.json();
    
    sqlx::query("UPDATE users SET email_verified = TRUE WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .unwrap();
    
    let login_body = json!({
        "email": test_email,
        "password": "password123"
    });
    
    let login_response = server
        .post("/auth/api/login")
        .json(&login_body)
        .await;
    
    let auth_response: AuthResponse = login_response.json();
    
    // Logout
    let response = server
        .post("/auth/api/logout")
        .authorization_bearer(&auth_response.access_token)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    // Try to access protected endpoint with the same token (should fail)
    let response = server
        .get("/auth/api/profile")
        .authorization_bearer(&auth_response.access_token)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}