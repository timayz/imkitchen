use evento::prelude::*;
use recipe::{
    create_recipe, list_shared_recipes, recipe_projection, share_recipe, CreateRecipeCommand,
    Ingredient, InstructionStep, RecipeDiscoveryFilters, ShareRecipeCommand,
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

    // Run SQLx migrations for read model tables
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    pool
}

/// Helper to create evento executor
async fn setup_evento_executor(pool: Pool<Sqlite>) -> evento::Sqlite {
    pool.into()
}

/// Create test user using proper evento commands
async fn create_test_user_for_tests(
    pool: &SqlitePool,
    executor: &evento::Sqlite,
    email: &str,
    tier: &str,
) -> String {
    use user::commands::{
        register_user, upgrade_subscription, RegisterUserCommand, UpgradeSubscriptionCommand,
    };

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

    // Process user projection to populate read model synchronously
    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    user_id
}

/// Helper to create a test recipe - returns recipe_id
/// IMPORTANT: Must call process_projections() AND set_recipe_tags() after creating recipes
#[allow(clippy::too_many_arguments)]
async fn create_test_recipe(
    pool: &SqlitePool,
    executor: &evento::Sqlite,
    user_id: &str,
    title: &str,
    _cuisine: Option<String>, // Will be set via set_recipe_tags after projection
    _dietary_tags: Vec<String>, // Will be set via set_recipe_tags after projection
    prep_time: Option<u32>,
    cook_time: Option<u32>,
) -> String {
    let command = CreateRecipeCommand {
        title: title.to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Test Ingredient".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Test instruction".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: prep_time,
        cook_time_min: cook_time,
        advance_prep_hours: None,
        serving_size: Some(2),
    };

    create_recipe(command, user_id, executor, pool, false)
        .await
        .expect("Failed to create recipe")
}

/// Set cuisine and dietary tags for a recipe (call AFTER process_projections)
/// Always updates even if tags are empty to ensure clean state
async fn set_recipe_tags(
    pool: &SqlitePool,
    recipe_id: &str,
    cuisine: Option<String>,
    dietary_tags: Vec<String>,
) {
    let dietary_json = serde_json::to_string(&dietary_tags).unwrap();
    sqlx::query("UPDATE recipes SET cuisine = ?1, dietary_tags = ?2 WHERE id = ?3")
        .bind(cuisine)
        .bind(dietary_json)
        .bind(recipe_id)
        .execute(pool)
        .await
        .unwrap();
}

/// Process all pending events in the projection
async fn process_projections(pool: &SqlitePool, executor: &evento::Sqlite) {
    recipe_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();
}

/// Helper to share a recipe and process projection
async fn share_recipe_helper(
    pool: &SqlitePool,
    executor: &evento::Sqlite,
    recipe_id: &str,
    user_id: &str,
) {
    // Verify recipe exists before sharing
    let exists: Option<String> = sqlx::query_scalar("SELECT id FROM recipes WHERE id = ?")
        .bind(recipe_id)
        .fetch_optional(pool)
        .await
        .unwrap();

    if exists.is_none() {
        panic!("Recipe {} not found in read model before sharing. This indicates projection didn't process creation event.", recipe_id);
    }

    let share_cmd = ShareRecipeCommand {
        recipe_id: recipe_id.to_string(),
        user_id: user_id.to_string(),
        shared: true,
    };
    share_recipe(share_cmd, executor, pool).await.unwrap();

    // Process projection synchronously
    recipe_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_list_shared_recipes_empty() {
    let pool = setup_test_db().await;

    let filters = RecipeDiscoveryFilters::default();
    let recipes = list_shared_recipes(&pool, filters)
        .await
        .expect("Query failed");

    assert_eq!(recipes.len(), 0);
}

#[tokio::test]
async fn test_list_shared_recipes_basic() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Create recipe
    let recipe_id = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Shared Recipe",
        Some("Italian".to_string()),
        vec!["vegetarian".to_string()],
        Some(20),
        Some(30),
    )
    .await;

    // Process creation events
    process_projections(&pool, &executor).await;

    // Set tags
    set_recipe_tags(
        &pool,
        &recipe_id,
        Some("Italian".to_string()),
        vec!["vegetarian".to_string()],
    )
    .await;

    // Share recipe
    share_recipe_helper(&pool, &executor, &recipe_id, &user1_id).await;

    // Query shared recipes
    let filters = RecipeDiscoveryFilters::default();
    let recipes = list_shared_recipes(&pool, filters)
        .await
        .expect("Query failed");

    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].title, "Shared Recipe");
    assert!(recipes[0].is_shared);
}

#[tokio::test]
async fn test_filter_by_cuisine() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Create multiple recipes with different cuisines
    let recipe1 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Italian Pasta",
        Some("Italian".to_string()),
        vec![],
        Some(10),
        Some(15),
    )
    .await;

    let recipe2 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Asian Stir Fry",
        Some("Asian".to_string()),
        vec![],
        Some(15),
        Some(10),
    )
    .await;

    // Process all creation events
    process_projections(&pool, &executor).await;

    // Set tags
    set_recipe_tags(&pool, &recipe1, Some("Italian".to_string()), vec![]).await;
    set_recipe_tags(&pool, &recipe2, Some("Asian".to_string()), vec![]).await;

    // Share both recipes
    share_recipe_helper(&pool, &executor, &recipe1, &user1_id).await;
    share_recipe_helper(&pool, &executor, &recipe2, &user1_id).await;

    // Filter by Italian cuisine
    let filters = RecipeDiscoveryFilters {
        cuisine: Some("Italian".to_string()),
        ..Default::default()
    };
    let recipes = list_shared_recipes(&pool, filters).await.unwrap();

    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].title, "Italian Pasta");
    assert_eq!(recipes[0].cuisine, Some("Italian".to_string()));
}

#[tokio::test]
async fn test_filter_by_prep_time() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Create recipes with different prep times
    let recipe1 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Quick Recipe",
        None,
        vec![],
        Some(10), // prep
        Some(15), // cook = 25 total
    )
    .await;

    let recipe2 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Slow Recipe",
        None,
        vec![],
        Some(30), // prep
        Some(60), // cook = 90 total
    )
    .await;

    // Process all creation events
    process_projections(&pool, &executor).await;

    // Share both
    share_recipe_helper(&pool, &executor, &recipe1, &user1_id).await;
    share_recipe_helper(&pool, &executor, &recipe2, &user1_id).await;

    // Filter by max 30 minutes total time
    let filters = RecipeDiscoveryFilters {
        max_prep_time: Some(30),
        ..Default::default()
    };
    let recipes = list_shared_recipes(&pool, filters).await.unwrap();

    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].title, "Quick Recipe");
}

#[tokio::test]
async fn test_filter_by_dietary() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Create recipes with different dietary tags
    let recipe1 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Vegan Salad",
        None,
        vec!["vegan".to_string(), "gluten-free".to_string()],
        Some(10),
        Some(0),
    )
    .await;

    let recipe2 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Meat Dish",
        None,
        vec![],
        Some(15),
        Some(30),
    )
    .await;

    // Process all creation events
    process_projections(&pool, &executor).await;

    // Set dietary tags
    set_recipe_tags(
        &pool,
        &recipe1,
        None,
        vec!["vegan".to_string(), "gluten-free".to_string()],
    )
    .await;
    set_recipe_tags(&pool, &recipe2, None, vec![]).await;

    // Share both
    share_recipe_helper(&pool, &executor, &recipe1, &user1_id).await;
    share_recipe_helper(&pool, &executor, &recipe2, &user1_id).await;

    // Filter by vegan
    let filters = RecipeDiscoveryFilters {
        dietary: Some("vegan".to_string()),
        ..Default::default()
    };
    let recipes = list_shared_recipes(&pool, filters).await.unwrap();

    // Debug: print what we got
    for recipe in &recipes {
        eprintln!(
            "Found recipe: {} with dietary_tags: {:?}",
            recipe.title, recipe.dietary_tags
        );
    }

    assert_eq!(
        recipes.len(),
        1,
        "Expected 1 vegan recipe but found {}",
        recipes.len()
    );
    assert_eq!(recipes[0].title, "Vegan Salad");
}

#[tokio::test]
async fn test_search_by_title() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Create recipes with searchable titles
    let recipe1 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Chocolate Cake",
        None,
        vec![],
        Some(20),
        Some(40),
    )
    .await;

    let recipe2 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Vanilla Cookies",
        None,
        vec![],
        Some(15),
        Some(12),
    )
    .await;

    // Process all creation events
    process_projections(&pool, &executor).await;

    // Share both
    share_recipe_helper(&pool, &executor, &recipe1, &user1_id).await;
    share_recipe_helper(&pool, &executor, &recipe2, &user1_id).await;

    // Search for "chocolate"
    let filters = RecipeDiscoveryFilters {
        search: Some("chocolate".to_string()),
        ..Default::default()
    };
    let recipes = list_shared_recipes(&pool, filters).await.unwrap();

    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].title, "Chocolate Cake");
}

#[tokio::test]
async fn test_sorting_alphabetical() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Create recipes in non-alphabetical order
    let recipe1 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Zebra Cake",
        None,
        vec![],
        Some(10),
        Some(20),
    )
    .await;
    let recipe2 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Apple Pie",
        None,
        vec![],
        Some(15),
        Some(30),
    )
    .await;
    let recipe3 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Mango Smoothie",
        None,
        vec![],
        Some(5),
        Some(0),
    )
    .await;

    // Process all creation events
    process_projections(&pool, &executor).await;

    // Share all
    share_recipe_helper(&pool, &executor, &recipe1, &user1_id).await;
    share_recipe_helper(&pool, &executor, &recipe2, &user1_id).await;
    share_recipe_helper(&pool, &executor, &recipe3, &user1_id).await;

    // Sort alphabetically
    let filters = RecipeDiscoveryFilters {
        sort: Some("alphabetical".to_string()),
        ..Default::default()
    };
    let recipes = list_shared_recipes(&pool, filters).await.unwrap();

    assert_eq!(recipes.len(), 3);
    assert_eq!(recipes[0].title, "Apple Pie");
    assert_eq!(recipes[1].title, "Mango Smoothie");
    assert_eq!(recipes[2].title, "Zebra Cake");
}

#[tokio::test]
async fn test_pagination() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "premium").await;

    // Create 25 recipes
    let mut recipe_ids = Vec::new();
    for i in 0..25 {
        let recipe_id = create_test_recipe(
            &pool,
            &executor,
            &user1_id,
            &format!("Recipe {}", i),
            None,
            vec![],
            Some(10),
            Some(10),
        )
        .await;
        recipe_ids.push(recipe_id);
    }

    // Process all creation events
    process_projections(&pool, &executor).await;

    // Share all
    for recipe_id in &recipe_ids {
        share_recipe_helper(&pool, &executor, recipe_id, &user1_id).await;
    }

    // Page 1 (should return 20)
    let filters = RecipeDiscoveryFilters {
        page: Some(1),
        ..Default::default()
    };
    let recipes_page1 = list_shared_recipes(&pool, filters).await.unwrap();
    assert_eq!(recipes_page1.len(), 20);

    // Page 2 (should return 5)
    let filters = RecipeDiscoveryFilters {
        page: Some(2),
        ..Default::default()
    };
    let recipes_page2 = list_shared_recipes(&pool, filters).await.unwrap();
    assert_eq!(recipes_page2.len(), 5);
}

#[tokio::test]
async fn test_sql_injection_prevention_cuisine() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Create a test recipe
    let recipe_id = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Test Recipe",
        Some("Italian".to_string()),
        vec![],
        Some(10),
        Some(20),
    )
    .await;

    // Process creation events
    process_projections(&pool, &executor).await;

    share_recipe_helper(&pool, &executor, &recipe_id, &user1_id).await;

    // Attempt SQL injection via cuisine filter
    let filters = RecipeDiscoveryFilters {
        cuisine: Some("Italian' OR '1'='1".to_string()),
        ..Default::default()
    };
    let recipes = list_shared_recipes(&pool, filters).await.unwrap();

    // Should return 0 recipes (injection prevented)
    assert_eq!(recipes.len(), 0);
}

#[tokio::test]
async fn test_sql_injection_prevention_search() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    let recipe_id = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Safe Recipe",
        None,
        vec![],
        Some(10),
        Some(20),
    )
    .await;

    // Process creation events
    process_projections(&pool, &executor).await;

    share_recipe_helper(&pool, &executor, &recipe_id, &user1_id).await;

    // Attempt SQL injection via search filter
    let filters = RecipeDiscoveryFilters {
        search: Some("'; DROP TABLE recipes; --".to_string()),
        ..Default::default()
    };

    // Should not panic or execute malicious SQL
    let result = list_shared_recipes(&pool, filters).await;
    assert!(result.is_ok());

    // Verify table still exists
    let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM recipes")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_combined_filters() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Create diverse recipes
    let recipe1 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Quick Italian Vegan Pasta",
        Some("Italian".to_string()),
        vec!["vegan".to_string()],
        Some(10),
        Some(15),
    )
    .await;

    let recipe2 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Slow Italian Meat Lasagna",
        Some("Italian".to_string()),
        vec![],
        Some(30),
        Some(60),
    )
    .await;

    let recipe3 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Quick Asian Vegan Stir Fry",
        Some("Asian".to_string()),
        vec!["vegan".to_string()],
        Some(10),
        Some(10),
    )
    .await;

    // Process all creation events
    process_projections(&pool, &executor).await;

    // Set tags
    set_recipe_tags(
        &pool,
        &recipe1,
        Some("Italian".to_string()),
        vec!["vegan".to_string()],
    )
    .await;
    set_recipe_tags(&pool, &recipe2, Some("Italian".to_string()), vec![]).await;
    set_recipe_tags(
        &pool,
        &recipe3,
        Some("Asian".to_string()),
        vec!["vegan".to_string()],
    )
    .await;

    // Share all
    share_recipe_helper(&pool, &executor, &recipe1, &user1_id).await;
    share_recipe_helper(&pool, &executor, &recipe2, &user1_id).await;
    share_recipe_helper(&pool, &executor, &recipe3, &user1_id).await;

    // Filter: Italian + Vegan + Under 30 mins
    let filters = RecipeDiscoveryFilters {
        cuisine: Some("Italian".to_string()),
        dietary: Some("vegan".to_string()),
        max_prep_time: Some(30),
        ..Default::default()
    };
    let recipes = list_shared_recipes(&pool, filters).await.unwrap();

    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].title, "Quick Italian Vegan Pasta");
}

#[tokio::test]
async fn test_only_shared_recipes_visible() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Create one shared and one private recipe
    let recipe1 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Shared Recipe",
        None,
        vec![],
        Some(10),
        Some(10),
    )
    .await;
    let _recipe2 = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "Private Recipe",
        None,
        vec![],
        Some(15),
        Some(15),
    )
    .await;

    // Process all creation events
    process_projections(&pool, &executor).await;

    // Share only recipe1
    share_recipe_helper(&pool, &executor, &recipe1, &user1_id).await;

    // Query all shared recipes
    let filters = RecipeDiscoveryFilters::default();
    let recipes = list_shared_recipes(&pool, filters).await.unwrap();

    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].title, "Shared Recipe");
    assert!(recipes[0].is_shared);
}

#[tokio::test]
async fn test_deleted_recipes_not_visible() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    let recipe_id = create_test_recipe(
        &pool,
        &executor,
        &user1_id,
        "To Delete",
        None,
        vec![],
        Some(10),
        Some(10),
    )
    .await;

    // Process creation events
    process_projections(&pool, &executor).await;

    // Share recipe
    share_recipe_helper(&pool, &executor, &recipe_id, &user1_id).await;

    // Verify it's visible
    let filters = RecipeDiscoveryFilters::default();
    let recipes = list_shared_recipes(&pool, filters.clone()).await.unwrap();
    assert_eq!(recipes.len(), 1);

    // Soft delete recipe by updating deleted_at directly (delete command is internal)
    sqlx::query("UPDATE recipes SET deleted_at = datetime('now') WHERE id = ?")
        .bind(&recipe_id)
        .execute(&pool)
        .await
        .unwrap();

    // Verify it's no longer visible in discovery
    let recipes = list_shared_recipes(&pool, filters).await.unwrap();
    assert_eq!(recipes.len(), 0);
}
