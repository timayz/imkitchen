use recipe::{
    create_recipe, CreateRecipeCommand, Ingredient, InstructionStep, RecipeAggregate, RecipeError,
};
use sqlx::{Pool, Sqlite, SqlitePool};

/// Helper to create in-memory SQLite database for testing
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Create users table for freemium limit checks
    sqlx::query(
        r#"
        CREATE TABLE users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            tier TEXT NOT NULL DEFAULT 'free',
            recipe_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create recipes table for read model
    sqlx::query(
        r#"
        CREATE TABLE recipes (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            title TEXT NOT NULL,
            ingredients TEXT NOT NULL,
            instructions TEXT NOT NULL,
            prep_time_min INTEGER,
            cook_time_min INTEGER,
            advance_prep_hours INTEGER,
            serving_size INTEGER,
            is_favorite INTEGER NOT NULL DEFAULT 0,
            is_shared INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

/// Helper to create in-memory evento executor for testing
async fn setup_evento_executor(pool: Pool<Sqlite>) -> evento::Sqlite {
    use evento::prelude::*;

    // Run evento migrations to create event store tables
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut *conn, &Plan::apply_all())
        .await
        .unwrap();

    // Create evento executor using From trait
    pool.clone().into()
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
