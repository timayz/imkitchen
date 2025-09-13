use sqlx::PgPool;
use uuid::Uuid;

use imkitchen::{
    models::user::{CreateUserRequest, SkillLevel},
    repositories::UserRepository,
};

async fn setup_test_repo() -> (UserRepository, PgPool) {
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
    
    let repo = UserRepository::new(pool.clone());
    (repo, pool)
}

async fn cleanup_test_user(pool: &PgPool, email: &str) {
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await;
}

#[tokio::test]
async fn test_create_user() {
    let (repo, pool) = setup_test_repo().await;
    let test_email = format!("test_create_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    let request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: Some(SkillLevel::Intermediate),
        household_size: Some(3),
        dietary_preferences: Some(vec!["vegan".to_string(), "gluten_free".to_string()]),
        kitchen_equipment: None,
    };
    
    let password_hash = "hashed_password".to_string();
    let verification_token = Some("verification_token".to_string());
    
    let result = repo.create(request, password_hash.clone(), verification_token.clone()).await;
    
    assert!(result.is_ok());
    let user = result.unwrap();
    
    assert_eq!(user.email, test_email);
    assert_eq!(user.password_hash, password_hash);
    assert_eq!(user.skill_level, "intermediate");
    assert_eq!(user.household_size, 3);
    assert_eq!(user.dietary_preferences, vec!["vegan", "gluten_free"]);
    assert_eq!(user.verification_token, verification_token);
    assert!(!user.email_verified);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_find_by_email() {
    let (repo, pool) = setup_test_repo().await;
    let test_email = format!("test_find_email_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Test finding non-existent user
    let result = repo.find_by_email(&test_email).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create a user
    let request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let created_user = repo.create(request, "hashed_password".to_string(), None).await.unwrap();
    
    // Test finding existing user
    let result = repo.find_by_email(&test_email).await;
    assert!(result.is_ok());
    let found_user = result.unwrap();
    assert!(found_user.is_some());
    
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.email, test_email);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_find_by_id() {
    let (repo, pool) = setup_test_repo().await;
    let test_email = format!("test_find_id_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Test finding non-existent user
    let random_id = Uuid::new_v4();
    let result = repo.find_by_id(random_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create a user
    let request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let created_user = repo.create(request, "hashed_password".to_string(), None).await.unwrap();
    
    // Test finding existing user
    let result = repo.find_by_id(created_user.id).await;
    assert!(result.is_ok());
    let found_user = result.unwrap();
    assert!(found_user.is_some());
    
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.email, test_email);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_find_by_verification_token() {
    let (repo, pool) = setup_test_repo().await;
    let test_email = format!("test_verification_{}@example.com", Uuid::new_v4());
    let verification_token = format!("token_{}", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Test finding with non-existent token
    let result = repo.find_by_verification_token(&verification_token).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create a user with verification token
    let request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let created_user = repo.create(request, "hashed_password".to_string(), Some(verification_token.clone())).await.unwrap();
    
    // Test finding with existing token
    let result = repo.find_by_verification_token(&verification_token).await;
    assert!(result.is_ok());
    let found_user = result.unwrap();
    assert!(found_user.is_some());
    
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.verification_token, Some(verification_token));
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_verify_email() {
    let (repo, pool) = setup_test_repo().await;
    let test_email = format!("test_verify_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Create a user with verification token
    let request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let created_user = repo.create(request, "hashed_password".to_string(), Some("token".to_string())).await.unwrap();
    
    // Verify the user is not verified initially
    assert!(!created_user.email_verified);
    assert!(created_user.verification_token.is_some());
    
    // Verify the email
    let result = repo.verify_email(created_user.id).await;
    assert!(result.is_ok());
    
    let verified_user = result.unwrap();
    assert!(verified_user.email_verified);
    assert!(verified_user.verification_token.is_none());
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_update_password() {
    let (repo, pool) = setup_test_repo().await;
    let test_email = format!("test_update_pass_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Create a user
    let request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let created_user = repo.create(request, "old_hash".to_string(), None).await.unwrap();
    assert_eq!(created_user.password_hash, "old_hash");
    
    // Update the password
    let new_hash = "new_password_hash".to_string();
    let result = repo.update_password(created_user.id, new_hash.clone()).await;
    assert!(result.is_ok());
    
    let updated_user = result.unwrap();
    assert_eq!(updated_user.password_hash, new_hash);
    assert_ne!(updated_user.updated_at, created_user.updated_at);
    
    // Cleanup
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_delete_user() {
    let (repo, pool) = setup_test_repo().await;
    let test_email = format!("test_delete_{}@example.com", Uuid::new_v4());
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    // Create a user
    let request = CreateUserRequest {
        email: test_email.clone(),
        password: "password123".to_string(),
        skill_level: None,
        household_size: None,
        dietary_preferences: None,
        kitchen_equipment: None,
    };
    
    let created_user = repo.create(request, "hashed_password".to_string(), None).await.unwrap();
    
    // Verify user exists
    let found = repo.find_by_id(created_user.id).await.unwrap();
    assert!(found.is_some());
    
    // Delete the user
    let result = repo.delete(created_user.id).await;
    assert!(result.is_ok());
    assert!(result.unwrap()); // Should return true for successful deletion
    
    // Verify user no longer exists
    let found = repo.find_by_id(created_user.id).await.unwrap();
    assert!(found.is_none());
    
    // Try to delete non-existent user
    let result = repo.delete(created_user.id).await;
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should return false for non-existent user
}