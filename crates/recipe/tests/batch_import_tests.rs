use recipe::{
    batch_import_recipes, BatchImportRecipe, BatchImportRecipesCommand, Ingredient,
    InstructionStep, RecipeError,
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

    // Process user projection synchronously for tests
    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    user_id
}

/// Helper to create valid batch import recipe
fn create_valid_batch_recipe(title: &str, recipe_type: &str) -> BatchImportRecipe {
    BatchImportRecipe {
        title: title.to_string(),
        recipe_type: recipe_type.to_string(),
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
    }
}

/// AC-6, AC-9: Test batch import with 3 valid recipes
/// Expected: 3 successful, 0 failed
#[tokio::test]
async fn test_batch_import_valid_recipes() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "user1@test.com").await;

    let command = BatchImportRecipesCommand {
        recipes: vec![
            create_valid_batch_recipe("Recipe 1", "appetizer"),
            create_valid_batch_recipe("Recipe 2", "main_course"),
            create_valid_batch_recipe("Recipe 3", "dessert"),
        ],
    };

    let result = batch_import_recipes(command, &user_id, &executor, &pool, false)
        .await
        .unwrap();

    assert_eq!(result.total_attempted, 3);
    assert_eq!(result.successful_recipe_ids.len(), 3);
    assert_eq!(result.failed_imports.len(), 0);
}

/// AC-8: Test free tier overflow rejection
/// User with 8 recipes attempts to import 3 → RecipeLimitExceeded error
#[tokio::test]
async fn test_batch_import_rejects_free_tier_overflow() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "user2@test.com").await;

    // Create 8 recipes manually to reach 8/10 limit
    for i in 0..8 {
        let recipe_cmd = recipe::CreateRecipeCommand {
            title: format!("Recipe {}", i),
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
        };
        recipe::create_recipe(recipe_cmd, &user_id, &executor, &pool, false)
            .await
            .unwrap();

        // Process recipe projection synchronously
        recipe::recipe_projection(pool.clone())
            .unsafe_oneshot(&executor)
            .await
            .unwrap();

        // Process user projection to update recipe_count
        user::user_projection(pool.clone())
            .unsafe_oneshot(&executor)
            .await
            .unwrap();
    }

    // Attempt to import 3 more recipes (would exceed 10 limit)
    let command = BatchImportRecipesCommand {
        recipes: vec![
            create_valid_batch_recipe("Recipe 9", "appetizer"),
            create_valid_batch_recipe("Recipe 10", "main_course"),
            create_valid_batch_recipe("Recipe 11", "dessert"),
        ],
    };

    let result = batch_import_recipes(command, &user_id, &executor, &pool, false).await;
    assert!(matches!(result, Err(RecipeError::RecipeLimitReached)));
}

/// AC-9, AC-12: Test partial failure scenario
/// 2 valid recipes, 1 invalid (missing title) → 2 successful, 1 failed with error message
#[tokio::test]
async fn test_batch_import_partial_failure() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "user3@test.com").await;

    let command = BatchImportRecipesCommand {
        recipes: vec![
            create_valid_batch_recipe("Valid Recipe 1", "appetizer"),
            BatchImportRecipe {
                title: "ab".to_string(), // Invalid: too short (< 3 chars)
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
            },
            create_valid_batch_recipe("Valid Recipe 2", "dessert"),
        ],
    };

    let result = batch_import_recipes(command, &user_id, &executor, &pool, false)
        .await
        .unwrap();

    assert_eq!(result.total_attempted, 3);
    assert_eq!(result.successful_recipe_ids.len(), 2);
    assert_eq!(result.failed_imports.len(), 1);

    // Check that failed import has index and error message
    let (index, error) = &result.failed_imports[0];
    assert_eq!(*index, 1); // Second recipe (0-indexed)
    assert!(error.contains("Title must be between 3 and 200 characters"));
}

/// AC-5: Test empty array validation
/// Empty array [] → validation error
#[tokio::test]
async fn test_batch_import_empty_array() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "user4@test.com").await;

    let command = BatchImportRecipesCommand { recipes: vec![] };

    let result = batch_import_recipes(command, &user_id, &executor, &pool, false).await;
    assert!(matches!(result, Err(RecipeError::ValidationError(_))));
    assert!(result.unwrap_err().to_string().contains("No recipes found"));
}

/// AC-6: Test invalid recipe_type validation
/// Invalid recipe_type → validation error
#[tokio::test]
async fn test_batch_import_invalid_recipe_type() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "user5@test.com").await;

    let command = BatchImportRecipesCommand {
        recipes: vec![BatchImportRecipe {
            title: "Invalid Recipe".to_string(),
            recipe_type: "invalid_type".to_string(), // Invalid type
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
        }],
    };

    let result = batch_import_recipes(command, &user_id, &executor, &pool, false)
        .await
        .unwrap();

    assert_eq!(result.total_attempted, 1);
    assert_eq!(result.successful_recipe_ids.len(), 0);
    assert_eq!(result.failed_imports.len(), 1);

    let (index, error) = &result.failed_imports[0];
    assert_eq!(*index, 0);
    assert!(error.contains("recipe_type"));
}

/// AC-7: Test missing required fields validation
/// Missing title and ingredients → validation errors per recipe
#[tokio::test]
async fn test_batch_import_missing_required_fields() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "user6@test.com").await;

    let command = BatchImportRecipesCommand {
        recipes: vec![
            BatchImportRecipe {
                title: "".to_string(), // Invalid: empty title
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
            },
            BatchImportRecipe {
                title: "Valid Title".to_string(),
                recipe_type: "appetizer".to_string(),
                ingredients: vec![], // Invalid: empty ingredients
                instructions: vec![InstructionStep {
                    step_number: 1,
                    instruction_text: "Add salt".to_string(),
                    timer_minutes: None,
                }],
                prep_time_min: Some(5),
                cook_time_min: Some(10),
                advance_prep_hours: None,
                serving_size: Some(4),
            },
        ],
    };

    let result = batch_import_recipes(command, &user_id, &executor, &pool, false)
        .await
        .unwrap();

    assert_eq!(result.total_attempted, 2);
    assert_eq!(result.successful_recipe_ids.len(), 0);
    assert_eq!(result.failed_imports.len(), 2);

    // Check both recipes failed with appropriate errors
    assert_eq!(result.failed_imports[0].0, 0);
    assert!(result.failed_imports[0].1.contains("Title"));

    assert_eq!(result.failed_imports[1].0, 1);
    assert!(result.failed_imports[1].1.contains("ingredient"));
}
