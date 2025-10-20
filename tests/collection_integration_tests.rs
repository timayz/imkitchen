use evento::prelude::*;
use recipe::{
    add_recipe_to_collection, collection_projection, create_collection, create_recipe,
    delete_collection, query_collection_by_id, query_collections_by_user,
    query_collections_for_recipe, query_recipes_by_collection, recipe_projection,
    remove_recipe_from_collection, update_collection, AddRecipeToCollectionCommand,
    CreateCollectionCommand, CreateRecipeCommand, DeleteCollectionCommand, Ingredient,
    InstructionStep, RecipeError, RemoveRecipeFromCollectionCommand, UpdateCollectionCommand,
};
use sqlx::{Pool, Sqlite, SqlitePool};

/// Helper to create test database with required tables
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Run evento migrations for event store tables
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut *conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    // Run SQLx migrations for read model tables (same as production)
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    pool
}

/// Helper to create evento executor
async fn setup_evento_executor(pool: Pool<Sqlite>) -> evento::Sqlite {
    pool.into()
}

/// Create test user for collection tests using proper evento commands
async fn create_test_user_for_tests(pool: &SqlitePool, executor: &evento::Sqlite, email: &str) -> String {
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

    // Process user projection to populate read model synchronously
    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    user_id
}

#[tokio::test]
async fn test_create_collection_integration_with_read_model_projection() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com").await;

    // Create collection
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

    // Query read model
    let collection = query_collection_by_id(&collection_id, &pool)
        .await
        .unwrap()
        .expect("Collection should exist");

    assert_eq!(collection.id, collection_id);
    assert_eq!(collection.user_id, user1_id);
    assert_eq!(collection.name, "Weeknight Favorites");
    assert_eq!(
        collection.description,
        Some("Quick meals for busy nights".to_string())
    );
    assert_eq!(collection.recipe_count, 0);
}

#[tokio::test]
async fn test_update_collection_integration() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com").await;

    // Create collection
    let create_command = CreateCollectionCommand {
        name: "Original Name".to_string(),
        description: Some("Original description".to_string()),
    };
    let collection_id = create_collection(create_command, &user1_id, &executor)
        .await
        .unwrap();

    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Update collection
    let update_command = UpdateCollectionCommand {
        collection_id: collection_id.clone(),
        user_id: user1_id.clone(),
        name: Some("Updated Name".to_string()),
        description: Some(Some("Updated description".to_string())),
    };
    update_collection(update_command, &executor, &pool)
        .await
        .unwrap();

    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify update
    let collection = query_collection_by_id(&collection_id, &pool)
        .await
        .unwrap()
        .expect("Collection should exist");

    assert_eq!(collection.name, "Updated Name");
    assert_eq!(
        collection.description,
        Some("Updated description".to_string())
    );
}

#[tokio::test]
async fn test_delete_collection_integration() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com").await;

    // Create collection
    let create_command = CreateCollectionCommand {
        name: "To Be Deleted".to_string(),
        description: None,
    };
    let collection_id = create_collection(create_command, &user1_id, &executor)
        .await
        .unwrap();

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

    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify soft delete - collection should not be returned by query (query filters out deleted)
    let result = query_collection_by_id(&collection_id, &pool).await.unwrap();
    assert!(
        result.is_none(),
        "Deleted collection should not be returned by query"
    );
}

#[tokio::test]
async fn test_add_recipe_to_collection_integration() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com").await;

    // Create collection
    let collection_command = CreateCollectionCommand {
        name: "Test Collection".to_string(),
        description: None,
    };
    let collection_id = create_collection(collection_command, &user1_id, &executor)
        .await
        .unwrap();

    // Create recipe
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

    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify assignment in read model
    let recipes = query_recipes_by_collection(&collection_id, &pool)
        .await
        .unwrap();
    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].id, recipe_id);

    let collections = query_collections_for_recipe(&recipe_id, &user1_id, &pool)
        .await
        .unwrap();
    assert_eq!(collections.len(), 1);
    assert_eq!(collections[0].id, collection_id);
}

#[tokio::test]
async fn test_remove_recipe_from_collection_integration() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com").await;

    // Create collection and recipe
    let collection_command = CreateCollectionCommand {
        name: "Test Collection".to_string(),
        description: None,
    };
    let collection_id = create_collection(collection_command, &user1_id, &executor)
        .await
        .unwrap();

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

    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Remove recipe from collection
    let remove_command = RemoveRecipeFromCollectionCommand {
        collection_id: collection_id.clone(),
        recipe_id: recipe_id.clone(),
        user_id: user1_id.clone(),
    };
    remove_recipe_from_collection(remove_command, &executor, &pool)
        .await
        .unwrap();

    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify removal
    let recipes = query_recipes_by_collection(&collection_id, &pool)
        .await
        .unwrap();
    assert_eq!(recipes.len(), 0, "Recipe should be removed from collection");
}

#[tokio::test]
async fn test_unauthorized_collection_access_returns_error() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com").await;
    let user2_id = create_test_user_for_tests(&pool, &executor, "user2@test.com").await;

    // User1 creates a collection
    let create_command = CreateCollectionCommand {
        name: "User1 Collection".to_string(),
        description: None,
    };
    let collection_id = create_collection(create_command, &user1_id, &executor)
        .await
        .unwrap();

    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User2 tries to delete user1's collection
    let delete_command = DeleteCollectionCommand {
        collection_id: collection_id.clone(),
        user_id: user2_id.clone(),
    };
    let result = delete_collection(delete_command, &executor, &pool).await;

    assert!(matches!(result, Err(RecipeError::PermissionDenied)));
}

#[tokio::test]
async fn test_query_collections_by_user() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com").await;
    let user2_id = create_test_user_for_tests(&pool, &executor, "user2@test.com").await;

    // User1 creates two collections
    let command1 = CreateCollectionCommand {
        name: "Collection 1".to_string(),
        description: None,
    };
    create_collection(command1, &user1_id, &executor)
        .await
        .unwrap();

    let command2 = CreateCollectionCommand {
        name: "Collection 2".to_string(),
        description: None,
    };
    create_collection(command2, &user1_id, &executor)
        .await
        .unwrap();

    // User2 creates one collection
    let command3 = CreateCollectionCommand {
        name: "Collection 3".to_string(),
        description: None,
    };
    create_collection(command3, &user2_id, &executor)
        .await
        .unwrap();

    collection_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Query user1's collections
    let user1_collections = query_collections_by_user(&user1_id, &pool).await.unwrap();
    assert_eq!(user1_collections.len(), 2);

    // Query user2's collections
    let user2_collections = query_collections_by_user(&user2_id, &pool).await.unwrap();
    assert_eq!(user2_collections.len(), 1);
}
