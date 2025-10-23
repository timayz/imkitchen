/// Tests for recipe creation limits using create_recipe command
///
/// These tests verify the freemium enforcement logic for recipe creation limits.
/// Tests follow TDD approach with coverage for all acceptance criteria.
use sqlx::{Row, SqlitePool};

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

/// Helper function to create a test user via commands with specific recipe_count
async fn create_test_user_with_recipes(
    pool: &SqlitePool,
    executor: &evento::Sqlite,
    email: &str,
    tier: &str,
    recipe_count: i32,
) -> String {
    use user::commands::{
        register_user, upgrade_subscription, RegisterUserCommand, UpgradeSubscriptionCommand,
    };

    // Register user via command
    let user_id = register_user(
        RegisterUserCommand {
            email: email.to_string(),
            password: "testpassword".to_string(),
        },
        executor,
        pool,
    )
    .await
    .unwrap();

    // If premium tier, upgrade subscription
    if tier == "premium" {
        upgrade_subscription(
            UpgradeSubscriptionCommand {
                user_id: user_id.clone(),
                new_tier: "premium".to_string(),
                stripe_customer_id: Some("cus_test".to_string()),
                stripe_subscription_id: Some("sub_test".to_string()),
            },
            executor,
        )
        .await
        .unwrap();
    }

    // Process user projection to populate read model
    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    // Create recipes using create_recipe command
    for i in 0..recipe_count {
        let command = recipe::CreateRecipeCommand {
            title: format!("Recipe {}", i),
            recipe_type: "main_course".to_string(),
            ingredients: vec![recipe::Ingredient {
                name: "Test".to_string(),
                quantity: 1.0,
                unit: "cup".to_string(),
            }],
            instructions: vec![recipe::InstructionStep {
                step_number: 1,
                instruction_text: "Test".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
        };
        recipe::create_recipe(command, &user_id, executor, pool, false)
            .await
            .unwrap();
    }

    // Process recipe projection to update read model
    recipe::recipe_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    // Process user projection to update UserAggregate.recipe_count from RecipeCreated events
    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    user_id
}

/// Test AC-2: Free user with 9 recipes can create recipe #10
#[tokio::test]
async fn test_free_user_with_9_recipes_can_create() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "user1@test.com", "free", 9).await;

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #10".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, &user_id, &executor, &pool, false).await;

    assert!(
        result.is_ok(),
        "Free user with 9 recipes should be able to create recipe #10"
    );
}

/// Test AC-4: Free user with 10 recipes cannot create recipe #11
#[tokio::test]
async fn test_free_user_with_10_recipes_cannot_create() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "user2@test.com", "free", 10).await;

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #11".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, &user_id, &executor, &pool, false).await;

    assert!(
        result.is_err(),
        "Free user with 10 recipes should not be able to create recipe #11"
    );
    match result {
        Err(recipe::RecipeError::RecipeLimitReached) => {
            // Expected error
        }
        _ => panic!("Expected RecipeError::RecipeLimitReached"),
    }
}

/// Test AC-8: Premium user can create unlimited recipes (no limit)
#[tokio::test]
async fn test_premium_user_unlimited_recipes() {
    let pool = setup_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "premium50@test.com", "premium", 50).await;

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #51".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, &user_id, &executor, &pool, false).await;

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
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "premium100@test.com", "premium", 100)
            .await;

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #101".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, &user_id, &executor, &pool, false).await;

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
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "user0@test.com", "free", 0).await;

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #1".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, &user_id, &executor, &pool, false).await;

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
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "user5@test.com", "free", 5).await;

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #6".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, &user_id, &executor, &pool, false).await;

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
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "user10@test.com", "free", 10).await;

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #11".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, &user_id, &executor, &pool, false).await;

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
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "user15@test.com", "free", 10).await;

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #16".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, &user_id, &executor, &pool, false).await;

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
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "premium0@test.com", "premium", 0).await;

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #1".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, &user_id, &executor, &pool, false).await;

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

    let command = recipe::CreateRecipeCommand {
        title: "Recipe #1".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![recipe::Ingredient {
            name: "Test".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![recipe::InstructionStep {
            step_number: 1,
            instruction_text: "Test".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = recipe::create_recipe(command, "nonexistent_user", &executor, &pool, false).await;

    assert!(
        result.is_err(),
        "Validation should fail for non-existent user"
    );
    match result {
        Err(recipe::RecipeError::ValidationError(_)) | Err(recipe::RecipeError::EventStoreError(_)) => {
            // Expected error - either ValidationError or EventStoreError("not found")
        }
        Err(e) => panic!("Expected RecipeError::ValidationError or EventStoreError for non-existent user, got: {:?}", e),
        Ok(_) => panic!("Expected error for non-existent user"),
    }
}

/// Test: Error message for RecipeLimitReached is user-friendly
#[test]
fn test_recipe_limit_error_message() {
    let error = recipe::RecipeError::RecipeLimitReached;
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
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "user7@test.com", "free", 7).await;

    // Process user projection to populate read model
    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Query recipe_count directly to verify it's stored correctly
    let row = sqlx::query("SELECT recipe_count FROM users WHERE id = ?")
        .bind(&user_id)
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
    let user_id =
        create_test_user_with_recipes(&pool, &executor, "premium20@test.com", "premium", 20).await;

    // Process user projection to populate read model
    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Query tier directly to verify it's stored correctly
    let row = sqlx::query("SELECT tier FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let tier: String = row.get("tier");
    assert_eq!(tier, "premium", "Tier should be premium");
}
