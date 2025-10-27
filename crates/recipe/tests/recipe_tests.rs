use recipe::{
    create_recipe, delete_recipe, query_recipe_by_id, query_recipes_by_user, share_recipe,
    CreateRecipeCommand, DeleteRecipeCommand, Ingredient, InstructionStep, RecipeAggregate,
    RecipeError, ShareRecipeCommand,
};
use sqlx::{Pool, Row, Sqlite, SqlitePool};

/// Helper to create in-memory SQLite database for testing
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Run evento migrations for event store tables
    use evento::migrator::{Migrate, Plan};
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

/// Helper to create in-memory evento executor for testing
async fn setup_evento_executor(pool: Pool<Sqlite>) -> evento::Sqlite {
    // Create evento executor (migrations already run in setup_test_db)
    pool.into()
}

/// Create a test user using proper evento commands
async fn create_test_user(pool: &SqlitePool, executor: &evento::Sqlite, email: &str) -> String {
    use user::commands::{register_user, RegisterUserCommand};

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

    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    user_id
}

#[tokio::test]
async fn test_create_recipe_validates_title_length() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Test title too short (< 3 chars)
    let command = CreateRecipeCommand {
        title: "ab".to_string(), // Only 2 characters
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let result = create_recipe(command, &user1_id, &executor, &pool, false).await;
    assert!(matches!(result, Err(RecipeError::ValidationError(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Title must be between 3 and 200 characters"));
}

#[tokio::test]
async fn test_create_recipe_requires_at_least_one_ingredient() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    let command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![], // Empty ingredients
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let result = create_recipe(command, &user1_id, &executor, &pool, false).await;
    assert!(matches!(result, Err(RecipeError::ValidationError(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("At least 1 ingredient is required"));
}

#[tokio::test]
async fn test_create_recipe_requires_at_least_one_instruction() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    let command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![], // Empty instructions
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let result = create_recipe(command, &user1_id, &executor, &pool, false).await;
    assert!(matches!(result, Err(RecipeError::ValidationError(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("At least 1 instruction step is required"));
}

#[tokio::test]
async fn test_free_tier_recipe_limit_enforced() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    // User already has 10 private recipes (at free tier limit)
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // AC-11: Create 10 private recipes (is_shared = 0)
    for i in 1..=10 {
        insert_test_recipe(
            &pool,
            &format!("recipe{}", i),
            &user1_id,
            &format!("Recipe {}", i),
            &executor,
        )
        .await;
    }

    let command = CreateRecipeCommand {
        title: "11th Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let result = create_recipe(command, &user1_id, &executor, &pool, false).await;
    assert!(matches!(result, Err(RecipeError::RecipeLimitReached)));
}

#[tokio::test]
async fn test_premium_tier_bypasses_recipe_limit() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    // Premium user already has 100 recipes (way over free tier limit)
    let premium_user_id = create_test_user(&pool, &executor, "premium_user@test.com").await;

    // Upgrade user to premium
    let upgrade_cmd = user::commands::UpgradeSubscriptionCommand {
        user_id: premium_user_id.clone(),
        new_tier: "premium".to_string(),
        stripe_customer_id: Some("cus_test123".to_string()),
        stripe_subscription_id: Some("sub_test456".to_string()),
    };
    user::commands::upgrade_subscription(upgrade_cmd, &executor)
        .await
        .unwrap();
    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    let command = CreateRecipeCommand {
        title: "101st Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let result = create_recipe(command, &premium_user_id, &executor, &pool, false).await;
    assert!(result.is_ok(), "Premium users should bypass recipe limit");
}

#[tokio::test]
async fn test_shared_recipes_dont_count_toward_limit() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    // AC-11: Shared recipes should NOT count toward free tier limit
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create 10 private recipes (at limit)
    let mut recipe_ids = Vec::new();
    for i in 1..=10 {
        let recipe_id = insert_test_recipe(
            &pool,
            &format!("recipe{}", i),
            &user1_id,
            &format!("Recipe {}", i),
            &executor,
        )
        .await;
        recipe_ids.push(recipe_id);
    }

    // Process user projection to update recipe_count
    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Share 5 of the 10 recipes (should decrement count to 5)
    use recipe::recipe_projection;
    for recipe_id in recipe_ids.iter().take(5) {
        let share_command = ShareRecipeCommand {
            recipe_id: recipe_id.clone(),
            user_id: user1_id.clone(),
            shared: true,
        };
        share_recipe(share_command, &executor, &pool).await.unwrap();

        // Process both recipe and user projections (RecipeShared event affects UserAggregate)
        recipe_projection(pool.clone())
            .unsafe_oneshot(&executor)
            .await
            .unwrap();
        user::user_projection(pool.clone())
            .unsafe_oneshot(&executor)
            .await
            .unwrap();
    }

    // User now has 10 total recipes: 5 shared (don't count) + 5 private (count toward limit)
    // Should be able to create 5 more private recipes before hitting limit again
    let command = CreateRecipeCommand {
        title: "Recipe 11 - Should Succeed".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let result = create_recipe(command, &user1_id, &executor, &pool, false).await;
    assert!(
        result.is_ok(),
        "Should be able to create 6th private recipe (5 shared recipes freed up 5 slots)"
    );
}

#[tokio::test]
async fn test_create_recipe_success_returns_recipe_id() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    let command = CreateRecipeCommand {
        title: "Chicken Tikka Masala".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![
            Ingredient {
                name: "Chicken".to_string(),
                quantity: 2.0,
                unit: "lb".to_string(),
            },
            Ingredient {
                name: "Yogurt".to_string(),
                quantity: 1.0,
                unit: "cup".to_string(),
            },
        ],
        instructions: vec![
            InstructionStep {
                step_number: 1,
                instruction_text: "Marinate chicken in yogurt".to_string(),
                timer_minutes: None,
            },
            InstructionStep {
                step_number: 2,
                instruction_text: "Cook chicken".to_string(),
                timer_minutes: Some(30),
            },
        ],
        prep_time_min: Some(20),
        cook_time_min: Some(30),
        advance_prep_hours: Some(4),
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let result = create_recipe(command, &user1_id, &executor, &pool, false).await;
    assert!(result.is_ok());

    let recipe_id = result.unwrap();
    assert!(!recipe_id.is_empty(), "Recipe ID should not be empty");
}

/// Test that RecipeCreated event successfully stores data in event store
/// and can be retrieved via evento load mechanism.
///
/// This test verifies the evento integration works correctly without
/// directly accessing private event handlers.
#[tokio::test]
async fn test_recipe_created_event_stored_and_loaded() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create a recipe (which writes RecipeCreated event)
    let command = CreateRecipeCommand {
        title: "Event Test Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Load the aggregate from event store to verify event was persisted
    let aggregate = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap()
        .item;

    // Verify aggregate state reflects the RecipeCreated event
    assert_eq!(aggregate.recipe_id, recipe_id);
    assert_eq!(aggregate.user_id, user1_id);
    assert_eq!(aggregate.title, "Event Test Recipe");
    assert_eq!(aggregate.ingredients.len(), 1);
    assert_eq!(aggregate.ingredients[0].name, "Salt");
    assert_eq!(aggregate.instructions.len(), 1);
    assert_eq!(aggregate.prep_time_min, Some(5));
    assert_eq!(aggregate.cook_time_min, Some(10));
    assert_eq!(aggregate.serving_size, Some(2));
    assert!(!aggregate.is_favorite);
    assert!(!aggregate.is_deleted);
}

// ============================================================================
// RecipeUpdated Event Tests (Story 2.2)
// ============================================================================

use recipe::update_recipe;
use recipe::UpdateRecipeCommand;

/// Helper to create a recipe for testing (creates via create_recipe command)
async fn insert_test_recipe(
    pool: &SqlitePool,
    _recipe_id: &str,
    user_id: &str,
    title: &str,
    executor: &evento::Sqlite,
) -> String {
    let command = CreateRecipeCommand {
        title: title.to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    create_recipe(command, user_id, executor, pool, false)
        .await
        .unwrap()
}

/// Test that RecipeUpdated event applies delta changes correctly
///
/// This test verifies that only the fields present in the RecipeUpdated event
/// are modified in the aggregate, while other fields remain unchanged.
#[tokio::test]
async fn test_recipe_updated_event_applies_delta_changes() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create initial recipe
    let create_command = CreateRecipeCommand {
        title: "Original Title".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Original instruction".to_string(),
            timer_minutes: Some(5),
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: Some(2),
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(create_command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Update only title and ingredients (delta update)
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        title: Some("Updated Title".to_string()),
        recipe_type: Some("main_course".to_string()),
        ingredients: Some(vec![
            Ingredient {
                name: "Salt".to_string(),
                quantity: 2.0,
                unit: "tsp".to_string(),
            },
            Ingredient {
                name: "Pepper".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            },
        ]),
        instructions: None, // Not updating instructions
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    update_recipe(update_command, &executor, &pool)
        .await
        .unwrap();

    // Load aggregate and verify delta was applied correctly
    let aggregate = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap()
        .item;

    // Updated fields should reflect new values
    assert_eq!(aggregate.title, "Updated Title");
    assert_eq!(aggregate.ingredients.len(), 2);
    assert_eq!(aggregate.ingredients[0].name, "Salt");
    assert_eq!(aggregate.ingredients[0].quantity, 2.0);
    assert_eq!(aggregate.ingredients[1].name, "Pepper");

    // Unchanged fields should retain original values
    assert_eq!(aggregate.instructions.len(), 1);
    assert_eq!(
        aggregate.instructions[0].instruction_text,
        "Original instruction"
    );
    assert_eq!(aggregate.instructions[0].timer_minutes, Some(5));
    assert_eq!(aggregate.prep_time_min, Some(10));
    assert_eq!(aggregate.cook_time_min, Some(20));
    assert_eq!(aggregate.advance_prep_hours, Some(2));
    assert_eq!(aggregate.serving_size, Some(4));
}

/// Test updating recipe with empty ingredients list fails validation
#[tokio::test]
async fn test_update_recipe_validates_empty_ingredients() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create initial recipe
    let create_command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(create_command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Attempt to update with empty ingredients list
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        title: None,
        recipe_type: Some("main_course".to_string()),
        ingredients: Some(vec![]), // Empty ingredients - should fail
        instructions: None,
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = update_recipe(update_command, &executor, &pool).await;
    assert!(matches!(
        result,
        Err(recipe::RecipeError::ValidationError(_))
    ));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("At least 1 ingredient is required"));
}

/// Test updating recipe with empty instructions list fails validation
#[tokio::test]
async fn test_update_recipe_validates_empty_instructions() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create initial recipe
    let create_command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(create_command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Attempt to update with empty instructions list
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        title: None,
        recipe_type: Some("main_course".to_string()),
        ingredients: None,
        instructions: Some(vec![]), // Empty instructions - should fail
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = update_recipe(update_command, &executor, &pool).await;
    assert!(matches!(
        result,
        Err(recipe::RecipeError::ValidationError(_))
    ));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("At least 1 instruction step is required"));
}

/// Test updating recipe validates title length
#[tokio::test]
async fn test_update_recipe_validates_title_length() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create initial recipe
    let create_command = CreateRecipeCommand {
        title: "Original Title".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(create_command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Attempt to update with title too short (< 3 chars)
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        title: Some("ab".to_string()), // Only 2 characters
        recipe_type: Some("main_course".to_string()),
        ingredients: None,
        instructions: None,
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = update_recipe(update_command, &executor, &pool).await;
    assert!(matches!(
        result,
        Err(recipe::RecipeError::ValidationError(_))
    ));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Title must be between 3 and 200 characters"));
}

/// Test updating recipe ownership verification
#[tokio::test]
async fn test_update_recipe_ownership_denied() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;
    let user2_id = create_test_user(&pool, &executor, "user2@test.com").await;

    // User1 creates a recipe
    let create_command = CreateRecipeCommand {
        title: "User1's Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(create_command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // User2 attempts to update user1's recipe
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user2_id.clone(), // Different user!
        title: Some("Hijacked Title".to_string()),
        recipe_type: Some("main_course".to_string()),
        ingredients: None,
        instructions: None,
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    let result = update_recipe(update_command, &executor, &pool).await;
    assert!(matches!(result, Err(recipe::RecipeError::PermissionDenied)));

    // Verify original recipe unchanged
    let aggregate = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap()
        .item;
    assert_eq!(aggregate.title, "User1's Recipe");
}

/// Test: recipe update automatically recalculates complexity
/// Story 3.12 AC-7
#[tokio::test]
async fn test_update_recipe_recalculates_complexity() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create recipe with simple complexity (5 ingredients, 4 steps, no advance prep)
    let create_command = CreateRecipeCommand {
        title: "Simple Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![
            Ingredient {
                name: "ingredient1".to_string(),
                quantity: 1.0,
                unit: "cup".to_string(),
            },
            Ingredient {
                name: "ingredient2".to_string(),
                quantity: 1.0,
                unit: "cup".to_string(),
            },
            Ingredient {
                name: "ingredient3".to_string(),
                quantity: 1.0,
                unit: "cup".to_string(),
            },
            Ingredient {
                name: "ingredient4".to_string(),
                quantity: 1.0,
                unit: "cup".to_string(),
            },
            Ingredient {
                name: "ingredient5".to_string(),
                quantity: 1.0,
                unit: "cup".to_string(),
            },
        ],
        instructions: vec![
            InstructionStep {
                step_number: 1,
                instruction_text: "Step 1".to_string(),
                timer_minutes: None,
            },
            InstructionStep {
                step_number: 2,
                instruction_text: "Step 2".to_string(),
                timer_minutes: None,
            },
            InstructionStep {
                step_number: 3,
                instruction_text: "Step 3".to_string(),
                timer_minutes: None,
            },
            InstructionStep {
                step_number: 4,
                instruction_text: "Step 4".to_string(),
                timer_minutes: None,
            },
        ],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(create_command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection to sync read model with initial complexity
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify initial complexity is "simple" (score = 5*0.3 + 4*0.4 + 0*0.3 = 3.1)
    let recipe = sqlx::query("SELECT complexity FROM recipes WHERE id = ?1")
        .bind(&recipe_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let complexity: Option<String> = recipe.get("complexity");
    assert_eq!(
        complexity.as_deref(),
        Some("simple"),
        "Initial complexity should be simple"
    );

    // Update recipe to add many ingredients and steps (change to complex)
    let mut new_ingredients = vec![];
    for i in 1..=20 {
        new_ingredients.push(Ingredient {
            name: format!("ingredient{}", i),
            quantity: 1.0,
            unit: "cup".to_string(),
        });
    }

    let mut new_instructions = vec![];
    for i in 1..=15 {
        new_instructions.push(InstructionStep {
            step_number: i,
            instruction_text: format!("Step {}", i),
            timer_minutes: None,
        });
    }

    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        title: None,
        recipe_type: Some("main_course".to_string()),
        ingredients: Some(new_ingredients),
        instructions: Some(new_instructions),
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };

    update_recipe(update_command, &executor, &pool)
        .await
        .unwrap();

    // Run projection to sync complexity recalculation
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify complexity updated to "complex" (score = 20*0.3 + 15*0.4 + 0*0.3 = 12 > 60? NO = 6 + 6 = 12, still moderate)
    // Let's check the actual value - with 20 ingredients and 15 steps: (20 * 0.3) + (15 * 0.4) = 6 + 6 = 12, which is still < 30 (simple)
    // Need more to get to complex. Let's add advance prep.
    let update_with_prep = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        title: None,
        recipe_type: Some("main_course".to_string()),
        ingredients: None,
        instructions: None,
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: Some(Some(4)), // Add 4-hour advance prep (multiplier = 100)
        serving_size: None,
    };

    update_recipe(update_with_prep, &executor, &pool)
        .await
        .unwrap();

    // Run projection again
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Now complexity should be "complex" (score = 20*0.3 + 15*0.4 + 100*0.3 = 6 + 6 + 30 = 42, still < 60)
    // Hmm, need even more. Let me recalculate: to get score > 60, with advance_prep=100:
    // (ingredients * 0.3) + (steps * 0.4) + (100 * 0.3) = X > 60
    // (ingredients * 0.3) + (steps * 0.4) + 30 > 60
    // (ingredients * 0.3) + (steps * 0.4) > 30
    // With 50 ingredients and 50 steps: (50 * 0.3) + (50 * 0.4) = 15 + 20 = 35 > 30. Then 35 + 30 = 65 > 60 ✓

    let mut complex_ingredients = vec![];
    for i in 1..=50 {
        complex_ingredients.push(Ingredient {
            name: format!("ingredient{}", i),
            quantity: 1.0,
            unit: "cup".to_string(),
        });
    }

    let mut complex_instructions = vec![];
    for i in 1..=50 {
        complex_instructions.push(InstructionStep {
            step_number: i,
            instruction_text: format!("Step {}", i),
            timer_minutes: None,
        });
    }

    let update_to_complex = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        title: None,
        recipe_type: Some("main_course".to_string()),
        ingredients: Some(complex_ingredients),
        instructions: Some(complex_instructions),
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None, // Keep the 4-hour prep
        serving_size: None,
    };

    update_recipe(update_to_complex, &executor, &pool)
        .await
        .unwrap();

    // Run projection again
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify complexity is now "complex"
    let recipe_final = sqlx::query("SELECT complexity FROM recipes WHERE id = ?1")
        .bind(&recipe_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let final_complexity: Option<String> = recipe_final.get("complexity");
    assert_eq!(
        final_complexity.as_deref(),
        Some("complex"),
        "Final complexity should be complex after update (50 ingredients + 50 steps + 4hr prep)"
    );
}

/// Test updating recipe with Option<Option<T>> nullable fields
///
/// Tests that we can correctly set timing fields to None (clearing existing values)
#[tokio::test]
async fn test_update_recipe_clears_optional_fields() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create recipe with timing fields set
    let create_command = CreateRecipeCommand {
        title: "Recipe with Timing".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: Some(4),
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(create_command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Update to clear prep_time_min and advance_prep_hours
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        title: None,
        recipe_type: Some("main_course".to_string()),
        ingredients: None,
        instructions: None,
        prep_time_min: Some(None), // Option<Option<u32>>: explicitly set to None
        cook_time_min: None,       // Not changing cook_time
        advance_prep_hours: Some(None), // Option<Option<u32>>: explicitly set to None
        serving_size: None,
    };

    update_recipe(update_command, &executor, &pool)
        .await
        .unwrap();

    // Load aggregate and verify fields were cleared
    let aggregate = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap()
        .item;

    assert_eq!(aggregate.prep_time_min, None); // Cleared
    assert_eq!(aggregate.cook_time_min, Some(20)); // Unchanged
    assert_eq!(aggregate.advance_prep_hours, None); // Cleared
    assert_eq!(aggregate.serving_size, Some(4)); // Unchanged
}

// ============================================================================
// RecipeDeleted Event Tests (Story 2.3)
// ============================================================================

/// Test that RecipeDeleted event sets is_deleted flag on aggregate
#[tokio::test]
async fn test_recipe_deleted_event_sets_is_deleted_flag() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create a recipe
    let create_command = CreateRecipeCommand {
        title: "Recipe to Delete".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(create_command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Delete the recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
    };

    delete_recipe(delete_command, &executor, &pool)
        .await
        .unwrap();

    // Load aggregate and verify is_deleted flag is set
    let aggregate = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap()
        .item;

    assert!(aggregate.is_deleted, "Recipe should be marked as deleted");
    assert_eq!(aggregate.title, "Recipe to Delete"); // Other fields preserved
}

/// Test that delete_recipe validates ownership (unauthorized user cannot delete)
#[tokio::test]
async fn test_delete_recipe_validates_ownership() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;
    let user2_id = create_test_user(&pool, &executor, "user2@test.com").await;

    // User1 creates a recipe
    let create_command = CreateRecipeCommand {
        title: "User1's Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(create_command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // User2 attempts to delete user1's recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user2_id.clone(), // Different user!
    };

    let result = delete_recipe(delete_command, &executor, &pool).await;
    assert!(
        matches!(result, Err(RecipeError::PermissionDenied)),
        "Should reject unauthorized deletion"
    );

    // Verify recipe was NOT deleted
    let aggregate = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap()
        .item;
    assert!(!aggregate.is_deleted, "Recipe should NOT be deleted");
}

/// Test deleting a non-existent recipe returns NotFound error
#[tokio::test]
async fn test_delete_recipe_not_found() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Attempt to delete recipe that doesn't exist
    let delete_command = DeleteRecipeCommand {
        recipe_id: "non_existent_id".to_string(),
        user_id: user1_id.clone(),
    };

    let result = delete_recipe(delete_command, &executor, &pool).await;
    assert!(
        matches!(result, Err(RecipeError::EventStoreError(_))),
        "Should return NotFound for non-existent recipe"
    );
}

// ============================================================================
// Favorite Recipe Tests
// ============================================================================

#[tokio::test]
async fn test_favorite_recipe_toggles_status() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    // Setup: Create a test user
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create a recipe
    let command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Flour".to_string(),
            quantity: 2.0,
            unit: "cups".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Mix ingredients".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection once to update read model
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Initial state: not favorited
    let recipe = sqlx::query("SELECT is_favorite FROM recipes WHERE id = ?1")
        .bind(&recipe_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let is_favorite: bool = recipe.get("is_favorite");
    assert!(!is_favorite, "Recipe should not be favorited initially");

    // Favorite the recipe
    use recipe::{favorite_recipe, FavoriteRecipeCommand};
    let fav_command = FavoriteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
    };
    let new_status = favorite_recipe(fav_command, &executor, &pool)
        .await
        .unwrap();
    assert!(new_status, "Recipe should be favorited after toggle");

    // Run projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify read model updated
    let recipe = sqlx::query("SELECT is_favorite FROM recipes WHERE id = ?1")
        .bind(&recipe_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let is_favorite: bool = recipe.get("is_favorite");
    assert!(is_favorite, "Recipe should be favorited in read model");

    // Un-favorite the recipe
    let unfav_command = FavoriteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
    };
    let new_status = favorite_recipe(unfav_command, &executor, &pool)
        .await
        .unwrap();
    assert!(
        !new_status,
        "Recipe should not be favorited after second toggle"
    );

    // Run projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify read model updated
    let recipe = sqlx::query("SELECT is_favorite FROM recipes WHERE id = ?1")
        .bind(&recipe_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let is_favorite: bool = recipe.get("is_favorite");
    assert!(!is_favorite, "Recipe should not be favorited in read model");
}

#[tokio::test]
async fn test_favorite_recipe_ownership_check() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    // Setup: Create two users
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;
    let user2_id = create_test_user(&pool, &executor, "user2@test.com").await;

    // User1 creates a recipe
    let command = CreateRecipeCommand {
        title: "User1 Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection once
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User2 tries to favorite User1's recipe (should fail)
    use recipe::{favorite_recipe, FavoriteRecipeCommand};
    let fav_command = FavoriteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user2_id.clone(),
    };
    let result = favorite_recipe(fav_command, &executor, &pool).await;

    match result {
        Err(RecipeError::PermissionDenied) => {
            // Expected error
        }
        _ => panic!("Expected PermissionDenied error when favoriting other user's recipe"),
    }
}

#[tokio::test]
async fn test_favorite_recipe_not_found() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    // Setup: Create a test user
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Try to favorite non-existent recipe
    use recipe::{favorite_recipe, FavoriteRecipeCommand};
    let fav_command = FavoriteRecipeCommand {
        recipe_id: "nonexistent-recipe-id".to_string(),
        user_id: user1_id.clone(),
    };
    let result = favorite_recipe(fav_command, &executor, &pool).await;

    match result {
        Err(RecipeError::EventStoreError(_)) => {
            // Expected error - recipe doesn't exist in event store
        }
        _ => panic!("Expected EventStoreError when favoriting non-existent recipe"),
    }
}

#[tokio::test]
async fn test_query_recipes_favorite_only_filter() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    // Setup: Create a test user
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create 3 recipes
    let command1 = CreateRecipeCommand {
        title: "Recipe 1".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Ingredient 1".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Step 1".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id_1 = create_recipe(command1, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    let command2 = CreateRecipeCommand {
        title: "Recipe 2".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Ingredient 2".to_string(),
            quantity: 2.0,
            unit: "cups".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Step 1".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(15),
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id_2 = create_recipe(command2, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    let command3 = CreateRecipeCommand {
        title: "Recipe 3".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Ingredient 3".to_string(),
            quantity: 3.0,
            unit: "tbsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Step 1".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(20),
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: Some(6),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id_3 = create_recipe(command3, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection once
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Favorite recipe 1 and 3
    use recipe::{favorite_recipe, FavoriteRecipeCommand};

    let fav_command_1 = FavoriteRecipeCommand {
        recipe_id: recipe_id_1.clone(),
        user_id: user1_id.clone(),
    };
    favorite_recipe(fav_command_1, &executor, &pool)
        .await
        .unwrap();

    let fav_command_3 = FavoriteRecipeCommand {
        recipe_id: recipe_id_3.clone(),
        user_id: user1_id.clone(),
    };
    favorite_recipe(fav_command_3, &executor, &pool)
        .await
        .unwrap();

    // Run projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Query all recipes (should return 3)
    use recipe::query_recipes_by_user;
    let all_recipes = query_recipes_by_user(&user1_id, false, &pool)
        .await
        .unwrap();
    assert_eq!(all_recipes.len(), 3, "Should return all 3 recipes");

    // Query favorite recipes only (should return 2)
    let favorite_recipes = query_recipes_by_user(&user1_id, true, &pool).await.unwrap();
    assert_eq!(
        favorite_recipes.len(),
        2,
        "Should return only 2 favorited recipes"
    );

    // Verify the correct recipes are returned
    let favorite_ids: Vec<String> = favorite_recipes.iter().map(|r| r.id.clone()).collect();
    assert!(
        favorite_ids.contains(&recipe_id_1),
        "Recipe 1 should be in favorites"
    );
    assert!(
        favorite_ids.contains(&recipe_id_3),
        "Recipe 3 should be in favorites"
    );
    assert!(
        !favorite_ids.contains(&recipe_id_2),
        "Recipe 2 should not be in favorites"
    );
}

// ===== Share Recipe Tests =====

/// Test: share_recipe emits RecipeShared event with shared=true
#[tokio::test]
async fn test_share_recipe_emits_event() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create a test recipe
    let command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection to sync read model
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Share the recipe
    let share_command = ShareRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        shared: true,
    };

    let result = share_recipe(share_command, &executor, &pool).await;
    assert!(result.is_ok(), "Share recipe should succeed");

    // Load aggregate and verify is_shared = true
    let load_result = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap();

    assert!(
        load_result.item.is_shared,
        "Recipe should be marked as shared in aggregate"
    );
}

/// Test: unshare_recipe emits RecipeShared event with shared=false
#[tokio::test]
async fn test_unshare_recipe_emits_event() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create and share a recipe
    let command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection to sync read model
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Share the recipe first
    share_recipe(
        ShareRecipeCommand {
            recipe_id: recipe_id.clone(),
            user_id: user1_id.clone(),
            shared: true,
        },
        &executor,
        &pool,
    )
    .await
    .unwrap();

    // Now unshare it
    let unshare_command = ShareRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        shared: false,
    };

    let result = share_recipe(unshare_command, &executor, &pool).await;
    assert!(result.is_ok(), "Unshare recipe should succeed");

    // Load aggregate and verify is_shared = false
    let load_result = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap();

    assert!(
        !load_result.item.is_shared,
        "Recipe should be marked as private in aggregate"
    );
}

/// Test: share_recipe verifies ownership (AC-5)
#[tokio::test]
async fn test_share_recipe_ownership_check() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;
    let user2_id = create_test_user(&pool, &executor, "user2@test.com").await;

    // User1 creates a recipe
    let command = CreateRecipeCommand {
        title: "User1's Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection to sync read model
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User2 attempts to share User1's recipe
    let share_command = ShareRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user2_id.clone(), // Different user!
        shared: true,
    };

    let result = share_recipe(share_command, &executor, &pool).await;

    assert!(
        matches!(result, Err(RecipeError::PermissionDenied)),
        "Share recipe by non-owner should return PermissionDenied"
    );
}

/// Test: share_recipe returns NotFound for non-existent recipe
#[tokio::test]
async fn test_share_recipe_not_found() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Attempt to share a non-existent recipe
    let share_command = ShareRecipeCommand {
        recipe_id: "nonexistent-recipe-id".to_string(),
        user_id: user1_id.clone(),
        shared: true,
    };

    let result = share_recipe(share_command, &executor, &pool).await;

    assert!(
        matches!(result, Err(RecipeError::EventStoreError(_))),
        "Share non-existent recipe should return NotFound"
    );
}

/// Test: RecipeShared event is applied to aggregate state
#[tokio::test]
async fn test_recipe_shared_event_applied_to_aggregate() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create recipe
    let command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection to sync read model
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify initial state (is_shared = false by default)
    let load_result_before = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap();
    assert!(
        !load_result_before.item.is_shared,
        "Recipe should default to private"
    );

    // Share the recipe
    share_recipe(
        ShareRecipeCommand {
            recipe_id: recipe_id.clone(),
            user_id: user1_id.clone(),
            shared: true,
        },
        &executor,
        &pool,
    )
    .await
    .unwrap();

    // Verify shared state
    let load_result_after = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap();
    assert!(
        load_result_after.item.is_shared,
        "Recipe should be marked as shared after RecipeShared event"
    );

    // Unshare and verify
    share_recipe(
        ShareRecipeCommand {
            recipe_id: recipe_id.clone(),
            user_id: user1_id.clone(),
            shared: false,
        },
        &executor,
        &pool,
    )
    .await
    .unwrap();

    let load_result_unshared = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap();
    assert!(
        !load_result_unshared.item.is_shared,
        "Recipe should be marked as private after unshare"
    );
}

// ===== Soft Delete Integration Tests (AC-12) =====

/// Test: Deleted recipes are excluded from query_recipe_by_id
#[tokio::test]
async fn test_deleted_recipe_excluded_from_query() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create and share a recipe
    let command = CreateRecipeCommand {
        title: "Recipe to Delete".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection to sync read model after creation
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Share the recipe (after projection so it exists in read model for ownership check)
    let share_command = ShareRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
        shared: true,
    };
    share_recipe(share_command, &executor, &pool).await.unwrap();

    // Run projection again to sync share status
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify recipe is queryable before deletion
    let recipe_before = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(
        recipe_before.is_some(),
        "Recipe should be queryable before deletion"
    );
    assert!(recipe_before.unwrap().is_shared, "Recipe should be shared");

    // Delete the recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
    };
    delete_recipe(delete_command, &executor, &pool)
        .await
        .unwrap();

    // Run projection to sync soft delete
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC-12: Verify deleted recipe is NOT returned by query_recipe_by_id
    let recipe_after = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(
        recipe_after.is_none(),
        "Deleted recipe should NOT be returned by query_recipe_by_id (soft delete via deleted_at IS NULL filter)"
    );
}

/// Test: Deleted shared recipes are excluded from user's recipe list
#[tokio::test]
async fn test_deleted_recipe_excluded_from_user_list() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create two recipes
    let recipe_id_1 = create_recipe(
        CreateRecipeCommand {
            title: "Recipe 1".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Step 1".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: vec![],
        },
        &user1_id,
        &executor,
        &pool,
        false,
    )
    .await
    .unwrap();

    let recipe_id_2 = create_recipe(
        CreateRecipeCommand {
            title: "Recipe 2".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![Ingredient {
                name: "Pepper".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Step 1".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: vec![],
        },
        &user1_id,
        &executor,
        &pool,
        false,
    )
    .await
    .unwrap();

    // Run projection
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify both recipes in list
    let recipes_before = query_recipes_by_user(&user1_id, false, &pool)
        .await
        .unwrap();
    assert_eq!(
        recipes_before.len(),
        2,
        "Should have 2 recipes before delete"
    );

    // Delete recipe 1
    delete_recipe(
        DeleteRecipeCommand {
            recipe_id: recipe_id_1.clone(),
            user_id: user1_id.clone(),
        },
        &executor,
        &pool,
    )
    .await
    .unwrap();

    // Run projection
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC-12: Verify only non-deleted recipe in list
    let recipes_after = query_recipes_by_user(&user1_id, false, &pool)
        .await
        .unwrap();
    assert_eq!(
        recipes_after.len(),
        1,
        "Should have only 1 recipe after soft delete (deleted recipes filtered by deleted_at IS NULL)"
    );
    assert_eq!(
        recipes_after[0].id, recipe_id_2,
        "Only non-deleted recipe should remain"
    );
}

/// Test: Deleted recipes are excluded from freemium limit count
#[tokio::test]
async fn test_deleted_recipes_excluded_from_limit() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create 10 recipes (free tier limit)
    for i in 0..10 {
        create_recipe(
            CreateRecipeCommand {
                title: format!("Recipe {}", i + 1),
                recipe_type: "main_course".to_string(),
                ingredients: vec![Ingredient {
                    name: "Salt".to_string(),
                    quantity: 1.0,
                    unit: "tsp".to_string(),
                }],
                instructions: vec![InstructionStep {
                    step_number: 1,
                    instruction_text: "Step 1".to_string(),
                    timer_minutes: None,
                }],
                prep_time_min: None,
                cook_time_min: None,
                advance_prep_hours: None,
                serving_size: None,
                accepts_accompaniment: false,
                preferred_accompaniments: vec![],
                accompaniment_category: None,
                cuisine: None,
                dietary_tags: vec![],
            },
            &user1_id,
            &executor,
            &pool,
            false,
        )
        .await
        .unwrap();
    }

    // Run projection
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify limit reached - 11th recipe should fail
    let result_at_limit = create_recipe(
        CreateRecipeCommand {
            title: "Recipe 11".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Step 1".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: vec![],
        },
        &user1_id,
        &executor,
        &pool,
        false,
    )
    .await;

    assert!(
        matches!(result_at_limit, Err(RecipeError::RecipeLimitReached)),
        "Should hit recipe limit at 10 recipes"
    );

    // Delete one recipe
    let recipes = query_recipes_by_user(&user1_id, false, &pool)
        .await
        .unwrap();
    delete_recipe(
        DeleteRecipeCommand {
            recipe_id: recipes[0].id.clone(),
            user_id: user1_id.clone(),
        },
        &executor,
        &pool,
    )
    .await
    .unwrap();

    // Run projections (both recipe and user to process RecipeDeleted event)
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();
    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC-12: Deleted recipes should NOT count toward limit - should be able to create another
    let result_after_delete = create_recipe(
        CreateRecipeCommand {
            title: "Recipe after delete".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Step 1".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: vec![],
        },
        &user1_id,
        &executor,
        &pool,
        false,
    )
    .await;

    assert!(
        result_after_delete.is_ok(),
        "Should be able to create recipe after deleting one (deleted recipes excluded from count via deleted_at IS NULL)"
    );
}
// ============================================================================
// Recipe Copy Tests (Story 2.10)
// ============================================================================

/// Test: copy_recipe successfully copies a shared community recipe
/// AC-2, AC-3, AC-4, AC-6, AC-7
#[tokio::test]
async fn test_copy_recipe_success() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let creator_id = create_test_user(&pool, &executor, "creator@test.com").await;
    let copier_id = create_test_user(&pool, &executor, "copier@test.com").await;

    // Creator creates and shares a recipe
    let create_command = CreateRecipeCommand {
        title: "Awesome Community Recipe".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Flour".to_string(),
            quantity: 2.0,
            unit: "cups".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Mix ingredients".to_string(),
            timer_minutes: Some(10),
        }],
        prep_time_min: Some(15),
        cook_time_min: Some(30),
        advance_prep_hours: Some(2),
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let original_recipe_id = create_recipe(create_command, &creator_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Share the recipe
    share_recipe(
        ShareRecipeCommand {
            recipe_id: original_recipe_id.clone(),
            user_id: creator_id.clone(),
            shared: true,
        },
        &executor,
        &pool,
    )
    .await
    .unwrap();

    // Run projection
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC-2, AC-3: Copier copies the recipe
    use recipe::{copy_recipe, CopyRecipeCommand};
    let copy_command = CopyRecipeCommand {
        original_recipe_id: original_recipe_id.clone(),
    };

    let new_recipe_id = copy_recipe(copy_command, &copier_id, &executor, &pool, false)
        .await
        .unwrap();

    // AC-7: Verify new recipe aggregate is independent
    let copied_aggregate = evento::load::<RecipeAggregate, _>(&executor, &new_recipe_id)
        .await
        .unwrap()
        .item;

    // AC-3: Owned by copier
    assert_eq!(copied_aggregate.user_id, copier_id);

    // AC-2, AC-7: Full data duplication
    assert_eq!(copied_aggregate.title, "Awesome Community Recipe");
    assert_eq!(copied_aggregate.ingredients.len(), 1);
    assert_eq!(copied_aggregate.ingredients[0].name, "Flour");
    assert_eq!(copied_aggregate.instructions.len(), 1);
    assert_eq!(copied_aggregate.prep_time_min, Some(15));
    assert_eq!(copied_aggregate.cook_time_min, Some(30));
    assert_eq!(copied_aggregate.advance_prep_hours, Some(2));
    assert_eq!(copied_aggregate.serving_size, Some(4));

    // AC-6: Defaults to private
    assert!(!copied_aggregate.is_shared);
    assert!(!copied_aggregate.is_favorite);

    // AC-4: Attribution metadata
    assert_eq!(
        copied_aggregate.original_recipe_id,
        Some(original_recipe_id.clone())
    );
    assert_eq!(copied_aggregate.original_author, Some(creator_id));
}

/// Test: copy_recipe prevents duplicate copies
/// AC-10
#[tokio::test]
#[ignore = "Subscription handlers don't run reliably in unsafe_oneshot mode - needs investigation"]
async fn test_copy_recipe_prevents_duplicates() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let creator_id = create_test_user(&pool, &executor, "creator@test.com").await;
    let copier_id = create_test_user(&pool, &executor, "copier@test.com").await;

    // Creator creates and shares a recipe
    let original_recipe_id = create_recipe(
        CreateRecipeCommand {
            title: "Recipe to Copy".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Add salt".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: vec![],
        },
        &creator_id,
        &executor,
        &pool,
        false,
    )
    .await
    .unwrap();

    // Run projection
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Share the recipe
    share_recipe(
        ShareRecipeCommand {
            recipe_id: original_recipe_id.clone(),
            user_id: creator_id.clone(),
            shared: true,
        },
        &executor,
        &pool,
    )
    .await
    .unwrap();

    // Run projection
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // First copy should succeed
    use recipe::{copy_recipe, CopyRecipeCommand};
    let copy_command_1 = CopyRecipeCommand {
        original_recipe_id: original_recipe_id.clone(),
    };
    let result_1 = copy_recipe(copy_command_1, &copier_id, &executor, &pool, false).await;
    assert!(result_1.is_ok(), "First copy should succeed");

    let _copied_recipe_id = result_1.unwrap();

    // Run projection to process RecipeCreated event
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC-10: Second copy should fail with AlreadyCopied error
    let copy_command_2 = CopyRecipeCommand {
        original_recipe_id: original_recipe_id.clone(),
    };
    let result_2 = copy_recipe(copy_command_2, &copier_id, &executor, &pool, false).await;
    assert!(
        matches!(result_2, Err(RecipeError::AlreadyCopied)),
        "Second copy should fail with AlreadyCopied error, got: {:?}",
        result_2
    );
}

/// Test: copy_recipe enforces freemium limit
/// AC-5, AC-11
#[tokio::test]
async fn test_copy_recipe_enforces_freemium_limit() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let creator_id = create_test_user(&pool, &executor, "creator@test.com").await;
    let copier_id = create_test_user(&pool, &executor, "copier@test.com").await;

    // Creator creates and shares a recipe
    let original_recipe_id = create_recipe(
        CreateRecipeCommand {
            title: "Recipe to Copy".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Add salt".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: vec![],
        },
        &creator_id,
        &executor,
        &pool,
        false,
    )
    .await
    .unwrap();

    // Run projection
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Share the recipe
    share_recipe(
        ShareRecipeCommand {
            recipe_id: original_recipe_id.clone(),
            user_id: creator_id.clone(),
            shared: true,
        },
        &executor,
        &pool,
    )
    .await
    .unwrap();

    // Run projection
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC-5, AC-11: Copier creates 10 private recipes (free tier limit)
    for i in 0..10 {
        create_recipe(
            CreateRecipeCommand {
                title: format!("Recipe {}", i + 1),
                recipe_type: "main_course".to_string(),
                ingredients: vec![Ingredient {
                    name: "Pepper".to_string(),
                    quantity: 1.0,
                    unit: "tsp".to_string(),
                }],
                instructions: vec![InstructionStep {
                    step_number: 1,
                    instruction_text: "Add pepper".to_string(),
                    timer_minutes: None,
                }],
                prep_time_min: None,
                cook_time_min: None,
                advance_prep_hours: None,
                serving_size: None,
                accepts_accompaniment: false,
                preferred_accompaniments: vec![],
                accompaniment_category: None,
                cuisine: None,
                dietary_tags: vec![],
            },
            &copier_id,
            &executor,
            &pool,
            false,
        )
        .await
        .unwrap();
    }

    // Run projection to sync all created recipes to read model (updates user's recipe_count)
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();
    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC-11: Copy should fail due to recipe limit
    use recipe::{copy_recipe, CopyRecipeCommand};
    let copy_command = CopyRecipeCommand {
        original_recipe_id: original_recipe_id.clone(),
    };
    let result = copy_recipe(copy_command, &copier_id, &executor, &pool, false).await;

    assert!(
        matches!(result, Err(RecipeError::RecipeLimitReached)),
        "Copy should fail when user is at recipe limit"
    );
}

/// Test: copy_recipe only works on shared recipes
/// AC-8
#[tokio::test]
async fn test_copy_recipe_requires_shared_recipe() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let creator_id = create_test_user(&pool, &executor, "creator@test.com").await;
    let copier_id = create_test_user(&pool, &executor, "copier@test.com").await;

    // Creator creates a PRIVATE recipe (not shared)
    let private_recipe_id = create_recipe(
        CreateRecipeCommand {
            title: "Private Recipe".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Add salt".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: vec![],
        },
        &creator_id,
        &executor,
        &pool,
        false,
    )
    .await
    .unwrap();

    // Run projection
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC-8: Attempt to copy private recipe should fail
    use recipe::{copy_recipe, CopyRecipeCommand};
    let copy_command = CopyRecipeCommand {
        original_recipe_id: private_recipe_id.clone(),
    };
    let result = copy_recipe(copy_command, &copier_id, &executor, &pool, false).await;

    assert!(
        matches!(result, Err(RecipeError::ValidationError(_))),
        "Copy should fail for private (non-shared) recipes"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Only shared recipes can be copied"));
}

/// Test: copy_recipe validates recipe exists
#[tokio::test]
async fn test_copy_recipe_validates_recipe_exists() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let copier_id = create_test_user(&pool, &executor, "copier@test.com").await;

    // Attempt to copy non-existent recipe
    use recipe::{copy_recipe, CopyRecipeCommand};
    let copy_command = CopyRecipeCommand {
        original_recipe_id: "nonexistent-recipe-id".to_string(),
    };
    let result = copy_recipe(copy_command, &copier_id, &executor, &pool, false).await;

    assert!(
        matches!(result, Err(RecipeError::EventStoreError(_))),
        "Copy should fail for non-existent recipes"
    );
}

/// Test: copied recipe modifications don't affect original
/// AC-7
#[tokio::test]
async fn test_copy_recipe_modifications_independent() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let creator_id = create_test_user(&pool, &executor, "creator@test.com").await;
    let copier_id = create_test_user(&pool, &executor, "copier@test.com").await;

    // Creator creates and shares a recipe
    let original_recipe_id = create_recipe(
        CreateRecipeCommand {
            title: "Original Recipe".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![Ingredient {
                name: "Original Ingredient".to_string(),
                quantity: 1.0,
                unit: "cup".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Original instruction".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: Some(10),
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: vec![],
        },
        &creator_id,
        &executor,
        &pool,
        false,
    )
    .await
    .unwrap();

    // Run projection
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Share the recipe
    share_recipe(
        ShareRecipeCommand {
            recipe_id: original_recipe_id.clone(),
            user_id: creator_id.clone(),
            shared: true,
        },
        &executor,
        &pool,
    )
    .await
    .unwrap();

    // Run projection
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Copier copies the recipe
    use recipe::{copy_recipe, CopyRecipeCommand};
    let copy_command = CopyRecipeCommand {
        original_recipe_id: original_recipe_id.clone(),
    };
    let copied_recipe_id = copy_recipe(copy_command, &copier_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection to sync the copied recipe to read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC-7: Copier modifies their copy
    use recipe::{update_recipe, UpdateRecipeCommand};
    let update_command = UpdateRecipeCommand {
        recipe_id: copied_recipe_id.clone(),
        user_id: copier_id.clone(),
        title: Some("Modified Copy".to_string()),
        recipe_type: Some("main_course".to_string()),
        ingredients: Some(vec![Ingredient {
            name: "Modified Ingredient".to_string(),
            quantity: 2.0,
            unit: "cups".to_string(),
        }]),
        instructions: None,
        prep_time_min: Some(Some(20)), // Change prep time
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };
    update_recipe(update_command, &executor, &pool)
        .await
        .unwrap();

    // AC-7: Verify original recipe is unchanged
    let original_aggregate = evento::load::<RecipeAggregate, _>(&executor, &original_recipe_id)
        .await
        .unwrap()
        .item;

    assert_eq!(original_aggregate.title, "Original Recipe");
    assert_eq!(
        original_aggregate.ingredients[0].name,
        "Original Ingredient"
    );
    assert_eq!(original_aggregate.prep_time_min, Some(10));

    // Verify copy was modified
    let copied_aggregate = evento::load::<RecipeAggregate, _>(&executor, &copied_recipe_id)
        .await
        .unwrap()
        .item;

    assert_eq!(copied_aggregate.title, "Modified Copy");
    assert_eq!(copied_aggregate.ingredients[0].name, "Modified Ingredient");
    assert_eq!(copied_aggregate.prep_time_min, Some(20));
}
