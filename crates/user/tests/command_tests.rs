/// Unit tests for user domain commands, focusing on validate_recipe_creation
///
/// These tests verify the freemium enforcement logic for recipe creation limits.
/// Tests follow TDD approach with coverage for all acceptance criteria.
use sqlx::{Row, SqlitePool};
use user::{validate_recipe_creation, UserError};

/// Helper function to create in-memory SQLite database for testing
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // Run SQLx migrations for read model tables (same as production)
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

    pool
}

/// Helper function to insert a test user
async fn insert_test_user(pool: &SqlitePool, user_id: &str, tier: &str, recipe_count: i32) {
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(format!("{}@example.com", user_id))
    .bind("hashed_password")
    .bind(tier)
    .bind(recipe_count)
    .bind("2025-01-01T00:00:00Z")
    .execute(pool)
    .await
    .unwrap();
}

/// Test AC-2: Free user with 9 recipes can create recipe #10 (validation passes)
#[tokio::test]
async fn test_free_user_with_9_recipes_can_create() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "user1", "free", 9).await;

    let result = validate_recipe_creation("user1", &pool).await;

    assert!(
        result.is_ok(),
        "Free user with 9 recipes should be able to create recipe #10"
    );
}

/// Test AC-4: Free user with 10 recipes cannot create recipe #11 (validation fails)
#[tokio::test]
async fn test_free_user_with_10_recipes_cannot_create() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "user1", "free", 10).await;

    let result = validate_recipe_creation("user1", &pool).await;

    assert!(
        result.is_err(),
        "Free user with 10 recipes should not be able to create recipe #11"
    );
    match result {
        Err(UserError::RecipeLimitReached) => {
            // Expected error
        }
        _ => panic!("Expected UserError::RecipeLimitReached"),
    }
}

/// Test AC-8: Premium user can create unlimited recipes (no limit)
#[tokio::test]
async fn test_premium_user_unlimited_recipes() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "premium_user", "premium", 50).await;

    let result = validate_recipe_creation("premium_user", &pool).await;

    assert!(
        result.is_ok(),
        "Premium user with 50 recipes should be able to create more"
    );
}

/// Test: Premium user with 100 recipes can still create (no limit)
#[tokio::test]
async fn test_premium_user_100_recipes() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "premium_user", "premium", 100).await;

    let result = validate_recipe_creation("premium_user", &pool).await;

    assert!(
        result.is_ok(),
        "Premium user with 100 recipes should be able to create more"
    );
}

/// Test: Free user with 0 recipes can create (well under limit)
#[tokio::test]
async fn test_free_user_with_0_recipes_can_create() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "user1", "free", 0).await;

    let result = validate_recipe_creation("user1", &pool).await;

    assert!(
        result.is_ok(),
        "Free user with 0 recipes should be able to create"
    );
}

/// Test: Free user with 5 recipes can create (under limit)
#[tokio::test]
async fn test_free_user_with_5_recipes_can_create() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "user1", "free", 5).await;

    let result = validate_recipe_creation("user1", &pool).await;

    assert!(
        result.is_ok(),
        "Free user with 5 recipes should be able to create"
    );
}

/// Test: Free user with exactly 10 recipes hits limit
#[tokio::test]
async fn test_free_user_exactly_at_limit() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "user1", "free", 10).await;

    let result = validate_recipe_creation("user1", &pool).await;

    assert!(
        result.is_err(),
        "Free user with exactly 10 recipes should hit limit"
    );
}

/// Test: Free user with 15 recipes (over limit) cannot create
#[tokio::test]
async fn test_free_user_over_limit() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "user1", "free", 15).await;

    let result = validate_recipe_creation("user1", &pool).await;

    assert!(
        result.is_err(),
        "Free user with 15 recipes should not be able to create"
    );
}

/// Test: Premium user with 0 recipes can create (baseline)
#[tokio::test]
async fn test_premium_user_with_0_recipes() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "premium_user", "premium", 0).await;

    let result = validate_recipe_creation("premium_user", &pool).await;

    assert!(
        result.is_ok(),
        "Premium user with 0 recipes should be able to create"
    );
}

/// Test: Validation fails for non-existent user
#[tokio::test]
async fn test_validation_fails_for_nonexistent_user() {
    let pool = setup_test_db().await;

    let result = validate_recipe_creation("nonexistent_user", &pool).await;

    assert!(
        result.is_err(),
        "Validation should fail for non-existent user"
    );
    match result {
        Err(UserError::ValidationError(_)) => {
            // Expected error
        }
        _ => panic!("Expected UserError::ValidationError for non-existent user"),
    }
}

/// Test: Error message for RecipeLimitReached is user-friendly
#[test]
fn test_recipe_limit_error_message() {
    let error = UserError::RecipeLimitReached;
    let error_msg = error.to_string();

    assert!(
        error_msg.contains("Recipe limit reached"),
        "Error message should mention recipe limit"
    );
    assert!(
        error_msg.contains("Upgrade to premium"),
        "Error message should include upgrade prompt"
    );
    assert!(
        error_msg.contains("unlimited recipes"),
        "Error message should mention unlimited recipes for premium"
    );
}

/// Test RecipeCreated event structure
#[test]
fn test_recipe_created_event() {
    use chrono::Utc;
    use user::events::RecipeCreated;

    let event = RecipeCreated {
        user_id: "user123".to_string(),
        title: "Chicken Tikka Masala".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(event.user_id, "user123");
    assert_eq!(event.title, "Chicken Tikka Masala");
    assert!(!event.created_at.is_empty());
}

/// Test RecipeDeleted event structure
#[test]
fn test_recipe_deleted_event() {
    use chrono::Utc;
    use user::events::RecipeDeleted;

    let event = RecipeDeleted {
        user_id: "user123".to_string(),
        deleted_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(event.user_id, "user123");
    assert!(!event.deleted_at.is_empty());
}

/// Integration test: Verify recipe_count is queried correctly
#[tokio::test]
async fn test_recipe_count_query() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "user1", "free", 7).await;

    // Query recipe_count directly to verify it's stored correctly
    let row = sqlx::query("SELECT recipe_count FROM users WHERE id = ?")
        .bind("user1")
        .fetch_one(&pool)
        .await
        .unwrap();

    let recipe_count: i32 = row.get("recipe_count");
    assert_eq!(recipe_count, 7, "Recipe count should be 7");
}

/// Integration test: Verify tier is queried correctly
#[tokio::test]
async fn test_tier_query() {
    let pool = setup_test_db().await;
    insert_test_user(&pool, "premium_user", "premium", 20).await;

    // Query tier directly to verify it's stored correctly
    let row = sqlx::query("SELECT tier FROM users WHERE id = ?")
        .bind("premium_user")
        .fetch_one(&pool)
        .await
        .unwrap();

    let tier: String = row.get("tier");
    assert_eq!(tier, "premium", "Tier should be premium");
}
