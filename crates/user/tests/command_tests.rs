/// Unit tests for user domain commands, focusing on validate_recipe_creation
///
/// These tests verify the freemium enforcement logic for recipe creation limits.
/// Tests follow TDD approach with coverage for all acceptance criteria.
use sqlx::{Row, SqlitePool};
use user::{validate_recipe_creation, UserError};

/// Helper function to create in-memory SQLite database for testing
async fn setup_test_db() -> SqlitePool {
    use evento::prelude::*;

    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // Run evento migrations for event store tables
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut *conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    // Run SQLx migrations for read model tables (same as production)
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

    pool
}

/// Helper function to create a test user via evento with specific user_id and recipe_count
async fn create_test_user_with_recipes(
    user_id: &str,
    tier: &str,
    recipe_count: i32,
    executor: &evento::Sqlite,
) {
    use user::events::UserCreated;

    // Create UserCreated event
    evento::save::<user::aggregate::UserAggregate>(user_id.to_string())
        .data(&UserCreated {
            email: format!("{}@example.com", user_id),
            password_hash: "hashed_password".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(executor)
        .await
        .unwrap();

    // If premium tier, emit SubscriptionUpgraded event
    if tier == "premium" {
        use user::events::SubscriptionUpgraded;
        evento::save::<user::aggregate::UserAggregate>(user_id.to_string())
            .data(&SubscriptionUpgraded {
                new_tier: "premium".to_string(),
                stripe_customer_id: Some("cus_test".to_string()),
                stripe_subscription_id: Some("sub_test".to_string()),
                upgraded_at: chrono::Utc::now().to_rfc3339(),
            })
            .unwrap()
            .metadata(&true)
            .unwrap()
            .commit(executor)
            .await
            .unwrap();
    }

    // Simulate recipe_count by emitting RecipeCreated events
    for i in 0..recipe_count {
        use recipe::events::RecipeCreated;
        evento::save::<recipe::aggregate::RecipeAggregate>(format!("recipe_{}", i))
            .data(&RecipeCreated {
                user_id: user_id.to_string(),
                title: format!("Recipe {}", i),
                ingredients: vec![],
                instructions: vec![],
                prep_time_min: None,
                cook_time_min: None,
                advance_prep_hours: None,
                serving_size: None,
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .unwrap()
            .metadata(&true)
            .unwrap()
            .commit(executor)
            .await
            .unwrap();
    }
}

/// Test AC-2: Free user with 9 recipes can create recipe #10 (validation passes)
#[tokio::test]
async fn test_free_user_with_9_recipes_can_create() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("user1", "free", 9, &executor).await;

    let result = validate_recipe_creation("user1", &executor).await;

    assert!(
        result.is_ok(),
        "Free user with 9 recipes should be able to create recipe #10"
    );
}

/// Test AC-4: Free user with 10 recipes cannot create recipe #11 (validation fails)
#[tokio::test]
async fn test_free_user_with_10_recipes_cannot_create() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("user1", "free", 10, &executor).await;

    let result = validate_recipe_creation("user1", &executor).await;

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
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("premium_user", "premium", 50, &executor).await;

    let result = validate_recipe_creation("premium_user", &executor).await;

    assert!(
        result.is_ok(),
        "Premium user with 50 recipes should be able to create more"
    );
}

/// Test: Premium user with 100 recipes can still create (no limit)
#[tokio::test]
async fn test_premium_user_100_recipes() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("premium_user", "premium",100, &executor).await;

    let result = validate_recipe_creation("premium_user", &executor).await;

    assert!(
        result.is_ok(),
        "Premium user with 100 recipes should be able to create more"
    );
}

/// Test: Free user with 0 recipes can create (well under limit)
#[tokio::test]
async fn test_free_user_with_0_recipes_can_create() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("user1", "free",0, &executor).await;

    let result = validate_recipe_creation("user1", &executor).await;

    assert!(
        result.is_ok(),
        "Free user with 0 recipes should be able to create"
    );
}

/// Test: Free user with 5 recipes can create (under limit)
#[tokio::test]
async fn test_free_user_with_5_recipes_can_create() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("user1", "free",5, &executor).await;

    let result = validate_recipe_creation("user1", &executor).await;

    assert!(
        result.is_ok(),
        "Free user with 5 recipes should be able to create"
    );
}

/// Test: Free user with exactly 10 recipes hits limit
#[tokio::test]
async fn test_free_user_exactly_at_limit() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("user1", "free", 10, &executor).await;

    let result = validate_recipe_creation("user1", &executor).await;

    assert!(
        result.is_err(),
        "Free user with exactly 10 recipes should hit limit"
    );
}

/// Test: Free user with 15 recipes (over limit) cannot create
#[tokio::test]
async fn test_free_user_over_limit() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("user1", "free",15, &executor).await;

    let result = validate_recipe_creation("user1", &executor).await;

    assert!(
        result.is_err(),
        "Free user with 15 recipes should not be able to create"
    );
}

/// Test: Premium user with 0 recipes can create (baseline)
#[tokio::test]
async fn test_premium_user_with_0_recipes() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("premium_user", "premium",0, &executor).await;

    let result = validate_recipe_creation("premium_user", &executor).await;

    assert!(
        result.is_ok(),
        "Premium user with 0 recipes should be able to create"
    );
}

/// Test: Validation fails for non-existent user
#[tokio::test]
async fn test_validation_fails_for_nonexistent_user() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();

    let result = validate_recipe_creation("nonexistent_user", &executor).await;

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
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("user1", "free",7, &executor).await;

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
    let executor: evento::Sqlite = pool.clone().into();
    create_test_user_with_recipes("premium_user", "premium",20, &executor).await;

    // Query tier directly to verify it's stored correctly
    let row = sqlx::query("SELECT tier FROM users WHERE id = ?")
        .bind("premium_user")
        .fetch_one(&pool)
        .await
        .unwrap();

    let tier: String = row.get("tier");
    assert_eq!(tier, "premium", "Tier should be premium");
}
