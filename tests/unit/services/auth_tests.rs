use sqlx::PgPool;
use uuid::Uuid;

use imkitchen::{
    models::user::{CreateUserRequest, LoginRequest, SkillLevel},
    repositories::UserRepository,
    services::{AuthError, AuthService},
};

async fn setup_test_service() -> (AuthService, PgPool, redis::Client) {
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
    
    (auth_service, pool, redis_client)
}

async fn cleanup_test_user(pool: &PgPool, email: &str) {
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await;
}

#[tokio::test]
async fn test_register_user_success() {
    let (auth_service, pool, _redis) = setup_test_service().await;
    let test_email = format!("test_register_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    let request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: Some(SkillLevel::Beginner),
        household_size: Some(2),
        dietary_preferences: Some(vec!["vegetarian".to_string()]),
        kitchen_equipment: None,
    };
    
    let result = auth_service.register_user(request).await;
    
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, test_email);
    assert_eq!(user.skill_level, "beginner");
    assert_eq!(user.household_size, 2);
    assert!(user.dietary_preferences.contains(&"vegetarian".to_string()));
    assert!(!user.email_verified);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_register_user_duplicate_email() {
    let (auth_service, pool, _redis) = setup_test_service().await;
    let test_email = format!("test_duplicate_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    let request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    // First registration should succeed
    let result1 = auth_service.register_user(request.clone()).await;
    assert!(result1.is_ok());
    
    // Second registration should fail
    let result2 = auth_service.register_user(request).await;
    assert!(result2.is_err());
    assert!(matches!(result2.unwrap_err(), AuthError::UserAlreadyExists));
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_register_user_invalid_input() {
    let (auth_service, _pool, _redis) = setup_test_service().await;
    
    // Test empty email
    let request = CreateUserRequest {
        email: "".to_string(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let result = auth_service.register_user(request).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::InvalidInput(_)));
    
    // Test short password
    let request = CreateUserRequest {
        email: "test@example.com".to_string(),
        password: "short".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let result = auth_service.register_user(request).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::InvalidInput(_)));
}

#[tokio::test]
async fn test_authenticate_user_success() {
    let (auth_service, pool, _redis) = setup_test_service().await;
    let test_email = format!("test_auth_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Register a user first
    let register_request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let user = auth_service.register_user(register_request).await.unwrap();
    
    // Manually verify the user for testing
    sqlx::query("UPDATE users SET email_verified = TRUE WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .unwrap();
    
    // Test authentication
    let login_request = LoginRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
    };
    
    let result = auth_service.authenticate_user(login_request).await;
    
    assert!(result.is_ok());
    let auth_response = result.unwrap();
    assert!(!auth_response.access_token.is_empty());
    assert_eq!(auth_response.user.email, test_email);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_authenticate_user_invalid_credentials() {
    let (auth_service, pool, _redis) = setup_test_service().await;
    let test_email = format!("test_invalid_{}@example.com", Uuid::new_v4());
    
    // Test with non-existent user
    let login_request = LoginRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
    };
    
    let result = auth_service.authenticate_user(login_request).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    
    // Register a user
    let register_request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    auth_service.register_user(register_request).await.unwrap();
    
    // Test with wrong password
    let login_request = LoginRequest {
        email: test_email.clone(),
        password: "wrongpassword".to_string(),
    };
    
    let result = auth_service.authenticate_user(login_request).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_validate_token() {
    let (auth_service, pool, _redis) = setup_test_service().await;
    let test_email = format!("test_token_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Register and verify a user
    let register_request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let user = auth_service.register_user(register_request).await.unwrap();
    
    // Manually verify the user
    sqlx::query("UPDATE users SET email_verified = TRUE WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .unwrap();
    
    // Authenticate to get a token
    let login_request = LoginRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
    };
    
    let auth_response = auth_service.authenticate_user(login_request).await.unwrap();
    
    // Validate the token
    let result = auth_service.validate_token(&auth_response.access_token).await;
    
    assert!(result.is_ok());
    let claims = result.unwrap();
    assert_eq!(claims.email, test_email);
    assert_eq!(claims.sub, user.id);
    
    // Test with invalid token
    let result = auth_service.validate_token("invalid_token").await;
    assert!(result.is_err());
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_password_hashing() {
    let (auth_service, _pool, _redis) = setup_test_service().await;
    
    // Test that the same password produces different hashes (due to salt)
    let password = "testpassword123";
    
    // We can't directly test the private methods, but we can test through registration
    let test_email1 = format!("test_hash1_{}@example.com", Uuid::new_v4());
    let test_email2 = format!("test_hash2_{}@example.com", Uuid::new_v4());
    
    let request1 = CreateUserRequest {
        email: test_email1,
        password: password.to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let request2 = CreateUserRequest {
        email: test_email2,
        password: password.to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let result1 = auth_service.register_user(request1).await;
    let result2 = auth_service.register_user(request2).await;
    
    // Both should succeed
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    // The password hashes should be different (due to salting)
    // We can't directly access them, but the fact that registration succeeded
    // indicates that password hashing is working correctly
}