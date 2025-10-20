use recipe::{
    add_recipe_to_collection, collection_projection, create_collection, create_recipe,
    delete_collection, recipe_projection, remove_recipe_from_collection,
    AddRecipeToCollectionCommand, CollectionAggregate, CreateCollectionCommand,
    CreateRecipeCommand, DeleteCollectionCommand, Ingredient, InstructionStep, RecipeError,
    RemoveRecipeFromCollectionCommand,
};
use sqlx::{Pool, Row, Sqlite, SqlitePool};

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

    // Register user via command (creates aggregate + events)
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

    // Process user projection to populate read model
    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    user_id
}

/// Test CollectionCreated event application via create_collection command
#[tokio::test]
async fn test_collection_created_event_sets_name_and_description() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    let command = CreateCollectionCommand {
        name: "Weeknight Favorites".to_string(),
        description: Some("Quick meals for busy nights".to_string()),
    };

    let collection_id = create_collection(command, &user1_id, &executor)
        .await
        .unwrap();

    // Process events synchronously into read model
    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify aggregate was created by loading it
    let aggregate = evento::load::<CollectionAggregate, _>(&executor, &collection_id)
        .await
        .unwrap()
        .item;

    assert_eq!(aggregate.collection_id, collection_id);
    assert_eq!(aggregate.user_id, user1_id);
    assert_eq!(aggregate.name, "Weeknight Favorites");
    assert_eq!(
        aggregate.description,
        Some("Quick meals for busy nights".to_string())
    );
    assert!(!aggregate.is_deleted);
}

/// Test collection name validation (min 3 chars, max 100 chars)
#[tokio::test]
async fn test_collection_name_validation_min_length() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Test name too short (< 3 chars)
    let command = CreateCollectionCommand {
        name: "ab".to_string(), // Only 2 characters
        description: None,
    };

    let result = create_collection(command, &user1_id, &executor).await;
    assert!(matches!(result, Err(RecipeError::ValidationError(_))));
}

#[tokio::test]
async fn test_collection_name_validation_max_length() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Test name too long (> 100 chars)
    let command = CreateCollectionCommand {
        name: "a".repeat(101), // 101 characters
        description: None,
    };

    let result = create_collection(command, &user1_id, &executor).await;
    assert!(matches!(result, Err(RecipeError::ValidationError(_))));
}

/// Test ownership verification (user can only delete own collections)
#[tokio::test]
async fn test_ownership_verification_prevents_unauthorized_deletion() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;
    let user2_id = create_test_user(&pool, &executor, "user2@test.com").await;

    // User1 creates a collection
    let command = CreateCollectionCommand {
        name: "User1 Collection".to_string(),
        description: None,
    };
    let collection_id = create_collection(command, &user1_id, &executor)
        .await
        .unwrap();

    // Process events synchronously into read model
    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User2 tries to delete user1's collection (should fail)
    let delete_command = DeleteCollectionCommand {
        collection_id: collection_id.clone(),
        user_id: user2_id.clone(),
    };

    let result = delete_collection(delete_command, &executor, &pool).await;
    assert!(matches!(result, Err(RecipeError::PermissionDenied)));
}

/// Test recipe assignment/unassignment to collections
#[tokio::test]
async fn test_recipe_assignment_and_unassignment() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create a collection
    let collection_command = CreateCollectionCommand {
        name: "Test Collection".to_string(),
        description: None,
    };
    let collection_id = create_collection(collection_command, &user1_id, &executor)
        .await
        .unwrap();

    // Create a recipe
    let recipe_command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Mix".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };
    let recipe_id = create_recipe(recipe_command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Process events synchronously into read model
    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Add recipe to collection
    let add_command = AddRecipeToCollectionCommand {
        collection_id: collection_id.clone(),
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
    };
    add_recipe_to_collection(add_command, &executor, &pool)
        .await
        .unwrap();

    // Process events synchronously into read model
    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify aggregate has recipe
    let aggregate = evento::load::<CollectionAggregate, _>(&executor, &collection_id)
        .await
        .unwrap()
        .item;
    assert!(aggregate.recipe_ids.contains(&recipe_id));

    // Remove recipe from collection
    let remove_command = RemoveRecipeFromCollectionCommand {
        collection_id: collection_id.clone(),
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
    };
    remove_recipe_from_collection(remove_command, &executor, &pool)
        .await
        .unwrap();

    // Verify aggregate no longer has recipe
    let aggregate = evento::load::<CollectionAggregate, _>(&executor, &collection_id)
        .await
        .unwrap()
        .item;
    assert!(
        !aggregate.recipe_ids.contains(&recipe_id),
        "Recipe should have been removed from aggregate, but still found. Aggregate recipe_ids: {:?}",
        aggregate.recipe_ids
    );
}

/// Test collection deletion preserves recipes
#[tokio::test]
async fn test_collection_deletion_preserves_recipes() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user(&pool, &executor, "user1@test.com").await;

    // Create a collection
    let collection_command = CreateCollectionCommand {
        name: "Test Collection".to_string(),
        description: None,
    };
    let collection_id = create_collection(collection_command, &user1_id, &executor)
        .await
        .unwrap();

    // Create a recipe
    let recipe_command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Mix".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
    };
    let recipe_id = create_recipe(recipe_command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Process events synchronously into read model
    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Add recipe to collection
    let add_command = AddRecipeToCollectionCommand {
        collection_id: collection_id.clone(),
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
    };
    add_recipe_to_collection(add_command, &executor, &pool)
        .await
        .unwrap();

    // Process events synchronously into read model
    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Delete collection
    let delete_command = DeleteCollectionCommand {
        collection_id: collection_id.clone(),
        user_id: user1_id.clone(),
    };
    delete_collection(delete_command, &executor, &pool)
        .await
        .unwrap();

    // Verify aggregate is deleted
    let aggregate = evento::load::<CollectionAggregate, _>(&executor, &collection_id)
        .await
        .unwrap()
        .item;
    assert!(aggregate.is_deleted);

    // Verify recipe still exists in database
    let recipe_exists = sqlx::query("SELECT COUNT(*) as count FROM recipes WHERE id = ?1")
        .bind(&recipe_id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .get::<i32, _>("count");
    assert_eq!(
        recipe_exists, 1,
        "Recipe should still exist after collection deletion"
    );
}
