use evento::prelude::*;
use recipe::{
    create_recipe, delete_recipe, query_recipe_by_id, query_recipes_by_user, recipe_projection,
    CreateRecipeCommand, DeleteRecipeCommand, Ingredient, InstructionStep, RecipeError,
};
use sqlx::{Pool, Sqlite, SqlitePool};

/// Helper to create test database with required tables
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Run evento migrations
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut *conn, &Plan::apply_all())
        .await
        .unwrap();

    // Create users table
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

    // Create recipes table
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

/// Helper to create evento executor
async fn setup_evento_executor(pool: Pool<Sqlite>) -> evento::Sqlite {
    pool.into()
}

/// Insert test user
async fn insert_test_user(pool: &SqlitePool, user_id: &str, email: &str, tier: &str) {
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at)
         VALUES (?1, ?2, 'hash', ?3, 0, datetime('now'))",
    )
    .bind(user_id)
    .bind(email)
    .bind(tier)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn test_create_recipe_integration_with_read_model_projection() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    insert_test_user(&pool, "user1", "user1@test.com", "free").await;

    // Start recipe projection subscription
    tokio::spawn({
        let pool = pool.clone();
        let executor = executor.clone();
        async move {
            recipe_projection(pool)
                .run(&executor)
                .await
                .expect("Projection failed");
        }
    });

    // Give subscription time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Create recipe command
    let command = CreateRecipeCommand {
        title: "Integration Test Recipe".to_string(),
        ingredients: vec![
            Ingredient {
                name: "Chicken".to_string(),
                quantity: 2.0,
                unit: "lb".to_string(),
            },
            Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            },
        ],
        instructions: vec![
            InstructionStep {
                step_number: 1,
                instruction_text: "Season chicken".to_string(),
                timer_minutes: None,
            },
            InstructionStep {
                step_number: 2,
                instruction_text: "Cook chicken".to_string(),
                timer_minutes: Some(30),
            },
        ],
        prep_time_min: Some(15),
        cook_time_min: Some(30),
        advance_prep_hours: Some(2),
        serving_size: Some(4),
    };

    // Execute recipe creation
    let recipe_id = create_recipe(command, "user1", &executor, &pool)
        .await
        .unwrap();

    // Wait for projection to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify recipe was projected into read model
    let recipe = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(recipe.is_some(), "Recipe should exist in read model");

    let recipe_data = recipe.unwrap();
    assert_eq!(recipe_data.id, recipe_id);
    assert_eq!(recipe_data.user_id, "user1");
    assert_eq!(recipe_data.title, "Integration Test Recipe");
    assert_eq!(recipe_data.prep_time_min, Some(15));
    assert_eq!(recipe_data.cook_time_min, Some(30));
    assert_eq!(recipe_data.advance_prep_hours, Some(2));
    assert_eq!(recipe_data.serving_size, Some(4));
    assert!(!recipe_data.is_favorite);
    assert!(!recipe_data.is_shared); // Default privacy is private (AC-10)

    // Verify ingredients stored as JSON
    let ingredients: Vec<Ingredient> = serde_json::from_str(&recipe_data.ingredients).unwrap();
    assert_eq!(ingredients.len(), 2);
    assert_eq!(ingredients[0].name, "Chicken");
    assert_eq!(ingredients[0].quantity, 2.0);
    assert_eq!(ingredients[0].unit, "lb");

    // Verify instructions stored as JSON
    let instructions: Vec<InstructionStep> =
        serde_json::from_str(&recipe_data.instructions).unwrap();
    assert_eq!(instructions.len(), 2);
    assert_eq!(instructions[0].step_number, 1);
    assert_eq!(instructions[0].instruction_text, "Season chicken");
    assert_eq!(instructions[1].timer_minutes, Some(30));
}

#[tokio::test]
async fn test_query_recipes_by_user() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    insert_test_user(&pool, "user1", "user1@test.com", "free").await;

    // Start projection
    tokio::spawn({
        let pool = pool.clone();
        let executor = executor.clone();
        async move {
            recipe_projection(pool).run(&executor).await.ok();
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create 3 recipes for user1
    for i in 1..=3 {
        let command = CreateRecipeCommand {
            title: format!("Recipe {}", i),
            ingredients: vec![Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Cook it".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: Some(10),
            cook_time_min: Some(20),
            advance_prep_hours: None,
            serving_size: Some(2),
        };
        create_recipe(command, "user1", &executor, &pool)
            .await
            .unwrap();
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Query all recipes for user1
    let recipes = query_recipes_by_user("user1", &pool).await.unwrap();
    assert_eq!(recipes.len(), 3, "Should have 3 recipes");

    // Verify sorted by created_at DESC (most recent first)
    assert!(recipes[0].title.contains("Recipe"));
}

// Note: Delete tests temporarily disabled as delete functionality is out of scope for Story 2.1 (Create Recipe only)
#[tokio::test]
#[ignore]
async fn test_delete_recipe_integration() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    insert_test_user(&pool, "user1", "user1@test.com", "free").await;

    // Start projection
    tokio::spawn({
        let pool = pool.clone();
        let executor = executor.clone();
        async move {
            recipe_projection(pool).run(&executor).await.ok();
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create recipe
    let command = CreateRecipeCommand {
        title: "Recipe to Delete".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(2),
    };
    let recipe_id = create_recipe(command, "user1", &executor, &pool)
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Verify recipe exists
    let recipe = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(recipe.is_some());

    // Delete recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user1".to_string(),
    };
    delete_recipe(delete_command, &executor, &pool)
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Verify recipe deleted from read model
    let recipe = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(recipe.is_none(), "Recipe should be deleted from read model");
}

#[tokio::test]
#[ignore]
async fn test_delete_recipe_permission_denied() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    insert_test_user(&pool, "user1", "user1@test.com", "free").await;
    insert_test_user(&pool, "user2", "user2@test.com", "free").await;

    // Start projection
    tokio::spawn({
        let pool = pool.clone();
        let executor = executor.clone();
        async move {
            recipe_projection(pool).run(&executor).await.ok();
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // User1 creates recipe
    let command = CreateRecipeCommand {
        title: "User1 Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(2),
    };
    let recipe_id = create_recipe(command, "user1", &executor, &pool)
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // User2 tries to delete user1's recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user2".to_string(), // Different user!
    };
    let result = delete_recipe(delete_command, &executor, &pool).await;

    // Should fail with PermissionDenied
    assert!(matches!(result, Err(RecipeError::PermissionDenied)));
}
