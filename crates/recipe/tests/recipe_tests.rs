use recipe::{
    create_recipe, CreateRecipeCommand, Ingredient, InstructionStep, RecipeAggregate, RecipeError,
};
use sqlx::{Pool, Sqlite, SqlitePool};

/// Helper to create in-memory SQLite database for testing
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Run evento migrations for event store tables
    use evento::prelude::*;
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

/// Insert a test user into the database
async fn insert_test_user(pool: &SqlitePool, user_id: &str, tier: &str, recipe_count: i32) {
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at)
         VALUES (?1, ?2, 'hash', ?3, ?4, datetime('now'))",
    )
    .bind(user_id)
    .bind(format!("{}@test.com", user_id))
    .bind(tier)
    .bind(recipe_count)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn test_create_recipe_validates_title_length() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    insert_test_user(&pool, "user1", "free", 0).await;

    // Test title too short (< 3 chars)
    let command = CreateRecipeCommand {
        title: "ab".to_string(), // Only 2 characters
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
    };

    let result = create_recipe(command, "user1", &executor, &pool).await;
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
    insert_test_user(&pool, "user1", "free", 0).await;

    let command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
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
    };

    let result = create_recipe(command, "user1", &executor, &pool).await;
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
    insert_test_user(&pool, "user1", "free", 0).await;

    let command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
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
    };

    let result = create_recipe(command, "user1", &executor, &pool).await;
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

    // User already has 10 recipes (at free tier limit)
    insert_test_user(&pool, "user1", "free", 10).await;

    let command = CreateRecipeCommand {
        title: "11th Recipe".to_string(),
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
    };

    let result = create_recipe(command, "user1", &executor, &pool).await;
    assert!(matches!(result, Err(RecipeError::RecipeLimitReached)));
}

#[tokio::test]
async fn test_premium_tier_bypasses_recipe_limit() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    // Premium user already has 100 recipes (way over free tier limit)
    insert_test_user(&pool, "premium_user", "premium", 100).await;

    let command = CreateRecipeCommand {
        title: "101st Recipe".to_string(),
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
    };

    let result = create_recipe(command, "premium_user", &executor, &pool).await;
    assert!(result.is_ok(), "Premium users should bypass recipe limit");
}

#[tokio::test]
async fn test_create_recipe_success_returns_recipe_id() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    insert_test_user(&pool, "user1", "free", 0).await;

    let command = CreateRecipeCommand {
        title: "Chicken Tikka Masala".to_string(),
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
    };

    let result = create_recipe(command, "user1", &executor, &pool).await;
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
    insert_test_user(&pool, "user1", "free", 0).await;

    // Create a recipe (which writes RecipeCreated event)
    let command = CreateRecipeCommand {
        title: "Event Test Recipe".to_string(),
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
    };

    let recipe_id = create_recipe(command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Load the aggregate from event store to verify event was persisted
    let aggregate = evento::load::<RecipeAggregate, _>(&executor, &recipe_id)
        .await
        .unwrap()
        .item;

    // Verify aggregate state reflects the RecipeCreated event
    assert_eq!(aggregate.recipe_id, recipe_id);
    assert_eq!(aggregate.user_id, "user1");
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

/// Helper to insert a recipe into the read model (for update tests)
async fn insert_test_recipe(pool: &SqlitePool, recipe_id: &str, user_id: &str, title: &str) {
    let ingredients_json = serde_json::to_string(&vec![Ingredient {
        name: "Salt".to_string(),
        quantity: 1.0,
        unit: "tsp".to_string(),
    }])
    .unwrap();

    let instructions_json = serde_json::to_string(&vec![InstructionStep {
        step_number: 1,
        instruction_text: "Add salt".to_string(),
        timer_minutes: None,
    }])
    .unwrap();

    sqlx::query(
        "INSERT INTO recipes (id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min, advance_prep_hours, serving_size, is_favorite, is_shared, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 0, datetime('now'), datetime('now'))",
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind(title)
    .bind(ingredients_json)
    .bind(instructions_json)
    .bind(10)
    .bind(20)
    .bind(2)
    .bind(4)
    .execute(pool)
    .await
    .unwrap();
}

/// Test that RecipeUpdated event applies delta changes correctly
///
/// This test verifies that only the fields present in the RecipeUpdated event
/// are modified in the aggregate, while other fields remain unchanged.
#[tokio::test]
async fn test_recipe_updated_event_applies_delta_changes() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    insert_test_user(&pool, "user1", "free", 0).await;

    // Create initial recipe
    let create_command = CreateRecipeCommand {
        title: "Original Title".to_string(),
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
    };

    let recipe_id = create_recipe(create_command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Insert recipe into read model for ownership check
    insert_test_recipe(&pool, &recipe_id, "user1", "Original Title").await;

    // Update only title and ingredients (delta update)
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user1".to_string(),
        title: Some("Updated Title".to_string()),
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
    insert_test_user(&pool, "user1", "free", 0).await;

    // Create initial recipe
    let create_command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
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
    };

    let recipe_id = create_recipe(create_command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Insert recipe into read model for ownership check
    insert_test_recipe(&pool, &recipe_id, "user1", "Test Recipe").await;

    // Attempt to update with empty ingredients list
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user1".to_string(),
        title: None,
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
    insert_test_user(&pool, "user1", "free", 0).await;

    // Create initial recipe
    let create_command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
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
    };

    let recipe_id = create_recipe(create_command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Insert recipe into read model for ownership check
    insert_test_recipe(&pool, &recipe_id, "user1", "Test Recipe").await;

    // Attempt to update with empty instructions list
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user1".to_string(),
        title: None,
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
    insert_test_user(&pool, "user1", "free", 0).await;

    // Create initial recipe
    let create_command = CreateRecipeCommand {
        title: "Original Title".to_string(),
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
    };

    let recipe_id = create_recipe(create_command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Insert recipe into read model for ownership check
    insert_test_recipe(&pool, &recipe_id, "user1", "Original Title").await;

    // Attempt to update with title too short (< 3 chars)
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user1".to_string(),
        title: Some("ab".to_string()), // Only 2 characters
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
    insert_test_user(&pool, "user1", "free", 0).await;
    insert_test_user(&pool, "user2", "free", 0).await;

    // User1 creates a recipe
    let create_command = CreateRecipeCommand {
        title: "User1's Recipe".to_string(),
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
    };

    let recipe_id = create_recipe(create_command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Insert recipe into read model for ownership check
    insert_test_recipe(&pool, &recipe_id, "user1", "User1's Recipe").await;

    // User2 attempts to update user1's recipe
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user2".to_string(), // Different user!
        title: Some("Hijacked Title".to_string()),
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

/// Test updating recipe with Option<Option<T>> nullable fields
///
/// Tests that we can correctly set timing fields to None (clearing existing values)
#[tokio::test]
async fn test_update_recipe_clears_optional_fields() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    insert_test_user(&pool, "user1", "free", 0).await;

    // Create recipe with timing fields set
    let create_command = CreateRecipeCommand {
        title: "Recipe with Timing".to_string(),
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
    };

    let recipe_id = create_recipe(create_command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Insert recipe into read model for ownership check
    insert_test_recipe(&pool, &recipe_id, "user1", "Recipe with Timing").await;

    // Update to clear prep_time_min and advance_prep_hours
    let update_command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user1".to_string(),
        title: None,
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

use recipe::delete_recipe;
use recipe::DeleteRecipeCommand;

/// Test that RecipeDeleted event sets is_deleted flag on aggregate
#[tokio::test]
async fn test_recipe_deleted_event_sets_is_deleted_flag() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    insert_test_user(&pool, "user1", "free", 0).await;

    // Create a recipe
    let create_command = CreateRecipeCommand {
        title: "Recipe to Delete".to_string(),
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
    };

    let recipe_id = create_recipe(create_command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Insert recipe into read model for ownership check
    insert_test_recipe(&pool, &recipe_id, "user1", "Recipe to Delete").await;

    // Delete the recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user1".to_string(),
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
    insert_test_user(&pool, "user1", "free", 0).await;
    insert_test_user(&pool, "user2", "free", 0).await;

    // User1 creates a recipe
    let create_command = CreateRecipeCommand {
        title: "User1's Recipe".to_string(),
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
    };

    let recipe_id = create_recipe(create_command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Insert recipe into read model for ownership check
    insert_test_recipe(&pool, &recipe_id, "user1", "User1's Recipe").await;

    // User2 attempts to delete user1's recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user2".to_string(), // Different user!
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
    insert_test_user(&pool, "user1", "free", 0).await;

    // Attempt to delete recipe that doesn't exist
    let delete_command = DeleteRecipeCommand {
        recipe_id: "non_existent_id".to_string(),
        user_id: "user1".to_string(),
    };

    let result = delete_recipe(delete_command, &executor, &pool).await;
    assert!(
        matches!(result, Err(RecipeError::NotFound)),
        "Should return NotFound for non-existent recipe"
    );
}
