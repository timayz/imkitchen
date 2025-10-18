use evento::prelude::{Migrate, Plan};
use shopping::aggregate::ShoppingListAggregate;
use shopping::commands::{mark_item_collected, reset_shopping_list};
use shopping::commands::{
    GenerateShoppingListCommand, MarkItemCollectedCommand, ResetShoppingListCommand,
};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

/// Helper to create a test database and executor
async fn setup_test_db() -> (SqlitePool, evento::Sqlite) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();

    // Run evento migrations for event store
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    // Run migrations for shopping list read models
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS shopping_lists (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            meal_plan_id TEXT NOT NULL,
            week_start_date TEXT NOT NULL,
            generated_at TEXT NOT NULL,
            updated_at TEXT
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS shopping_list_items (
            id TEXT PRIMARY KEY,
            shopping_list_id TEXT NOT NULL,
            ingredient_name TEXT NOT NULL,
            quantity REAL NOT NULL,
            unit TEXT NOT NULL,
            category TEXT NOT NULL,
            is_collected INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (shopping_list_id) REFERENCES shopping_lists (id)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create evento executor
    let executor: evento::Sqlite = pool.clone().into();

    (pool, executor)
}

/// Test marking an item as collected
#[tokio::test]
async fn test_mark_item_collected() {
    // Setup: Create test database and executor
    let (_pool, executor) = setup_test_db().await;

    // Create a shopping list first
    let shopping_list_id = shopping::commands::generate_shopping_list(
        GenerateShoppingListCommand {
            user_id: "user-1".to_string(),
            meal_plan_id: "meal-plan-1".to_string(),
            week_start_date: "2025-10-20".to_string(), // Monday
            ingredients: vec![
                ("tomatoes".to_string(), 2.0, "lbs".to_string()),
                ("cheese".to_string(), 1.0, "lb".to_string()),
            ],
        },
        &executor,
    )
    .await
    .unwrap();

    // Get item ID (format: {shopping_list_id}-{index})
    let item_id = format!("{}-0", shopping_list_id);

    // Execute: Mark item as collected
    let cmd = MarkItemCollectedCommand {
        shopping_list_id: shopping_list_id.clone(),
        item_id: item_id.clone(),
        is_collected: true,
    };

    let result = mark_item_collected(cmd, &executor).await;
    assert!(result.is_ok(), "Mark item collected should succeed");

    // Verify: Load aggregate and check event was emitted
    let loaded = evento::load::<ShoppingListAggregate, _>(&executor, &shopping_list_id)
        .await
        .unwrap();

    // The aggregate doesn't store is_collected state, but we can verify the event was emitted
    // by checking that the version incremented (generation event + collected event = version 2)
    assert_eq!(
        loaded.event.version, 2,
        "Version should be 2 after two events"
    );
}

/// Test marking an item as uncollected (toggle from collected to uncollected)
#[tokio::test]
async fn test_mark_item_uncollected() {
    // Setup: Create test database and executor
    let (_pool, executor) = setup_test_db().await;

    // Create a shopping list
    let shopping_list_id = shopping::commands::generate_shopping_list(
        GenerateShoppingListCommand {
            user_id: "user-1".to_string(),
            meal_plan_id: "meal-plan-1".to_string(),
            week_start_date: "2025-10-20".to_string(),
            ingredients: vec![("bread".to_string(), 1.0, "loaf".to_string())],
        },
        &executor,
    )
    .await
    .unwrap();

    let item_id = format!("{}-0", shopping_list_id);

    // First, mark as collected
    mark_item_collected(
        MarkItemCollectedCommand {
            shopping_list_id: shopping_list_id.clone(),
            item_id: item_id.clone(),
            is_collected: true,
        },
        &executor,
    )
    .await
    .unwrap();

    // Execute: Mark as uncollected
    let cmd = MarkItemCollectedCommand {
        shopping_list_id: shopping_list_id.clone(),
        item_id: item_id.clone(),
        is_collected: false,
    };

    let result = mark_item_collected(cmd, &executor).await;
    assert!(result.is_ok(), "Mark item uncollected should succeed");

    // Verify: Version should be 3 (generate + collect + uncollect)
    let loaded = evento::load::<ShoppingListAggregate, _>(&executor, &shopping_list_id)
        .await
        .unwrap();
    assert_eq!(
        loaded.event.version, 3,
        "Version should be 3 after three events"
    );
}

/// Test reset shopping list
#[tokio::test]
async fn test_reset_shopping_list() {
    // Setup: Create test database and executor
    let (_pool, executor) = setup_test_db().await;

    // Create a shopping list with multiple items
    let shopping_list_id = shopping::commands::generate_shopping_list(
        GenerateShoppingListCommand {
            user_id: "user-1".to_string(),
            meal_plan_id: "meal-plan-1".to_string(),
            week_start_date: "2025-10-20".to_string(),
            ingredients: vec![
                ("apples".to_string(), 3.0, "lbs".to_string()),
                ("milk".to_string(), 1.0, "gallon".to_string()),
                ("eggs".to_string(), 12.0, "count".to_string()),
            ],
        },
        &executor,
    )
    .await
    .unwrap();

    // Mark some items as collected
    mark_item_collected(
        MarkItemCollectedCommand {
            shopping_list_id: shopping_list_id.clone(),
            item_id: format!("{}-0", shopping_list_id),
            is_collected: true,
        },
        &executor,
    )
    .await
    .unwrap();

    mark_item_collected(
        MarkItemCollectedCommand {
            shopping_list_id: shopping_list_id.clone(),
            item_id: format!("{}-1", shopping_list_id),
            is_collected: true,
        },
        &executor,
    )
    .await
    .unwrap();

    // Execute: Reset shopping list
    let cmd = ResetShoppingListCommand {
        shopping_list_id: shopping_list_id.clone(),
    };

    let result = reset_shopping_list(cmd, &executor).await;
    assert!(result.is_ok(), "Reset shopping list should succeed");

    // Verify: Version should be 4 (generate + 2 collects + reset)
    let loaded = evento::load::<ShoppingListAggregate, _>(&executor, &shopping_list_id)
        .await
        .unwrap();
    assert_eq!(
        loaded.event.version, 4,
        "Version should be 4 after four events"
    );
}

/// Test evento aggregate event handler for ShoppingListItemCollected
#[tokio::test]
async fn test_shopping_list_item_collected_aggregate_handler() {
    // Setup: Create test database and executor
    let (_pool, executor) = setup_test_db().await;

    // Create a shopping list
    let shopping_list_id = shopping::commands::generate_shopping_list(
        GenerateShoppingListCommand {
            user_id: "user-1".to_string(),
            meal_plan_id: "meal-plan-1".to_string(),
            week_start_date: "2025-10-20".to_string(),
            ingredients: vec![("lettuce".to_string(), 1.0, "head".to_string())],
        },
        &executor,
    )
    .await
    .unwrap();

    // Mark item as collected
    mark_item_collected(
        MarkItemCollectedCommand {
            shopping_list_id: shopping_list_id.clone(),
            item_id: format!("{}-0", shopping_list_id),
            is_collected: true,
        },
        &executor,
    )
    .await
    .unwrap();

    // Verify: Load aggregate (evento will replay events including shopping_list_item_collected)
    let loaded = evento::load::<ShoppingListAggregate, _>(&executor, &shopping_list_id)
        .await
        .unwrap();

    // The aggregate should successfully rebuild from events (no panics/errors)
    assert_eq!(loaded.item.shopping_list_id, shopping_list_id);
    assert_eq!(loaded.item.user_id, "user-1");

    // Aggregate doesn't store is_collected state (managed in read model)
    // But we can verify that the event was applied by checking version
    assert_eq!(loaded.event.version, 2);
}

/// Test evento aggregate event handler for ShoppingListReset
#[tokio::test]
async fn test_shopping_list_reset_aggregate_handler() {
    // Setup: Create test database and executor
    let (_pool, executor) = setup_test_db().await;

    // Create a shopping list
    let shopping_list_id = shopping::commands::generate_shopping_list(
        GenerateShoppingListCommand {
            user_id: "user-1".to_string(),
            meal_plan_id: "meal-plan-1".to_string(),
            week_start_date: "2025-10-20".to_string(),
            ingredients: vec![("onions".to_string(), 2.0, "lbs".to_string())],
        },
        &executor,
    )
    .await
    .unwrap();

    // Reset shopping list
    reset_shopping_list(
        ResetShoppingListCommand {
            shopping_list_id: shopping_list_id.clone(),
        },
        &executor,
    )
    .await
    .unwrap();

    // Verify: Load aggregate (evento will replay events including shopping_list_reset)
    let loaded = evento::load::<ShoppingListAggregate, _>(&executor, &shopping_list_id)
        .await
        .unwrap();

    // The aggregate should successfully rebuild from events
    assert_eq!(loaded.item.shopping_list_id, shopping_list_id);
    assert_eq!(loaded.event.version, 2); // generate + reset
}

/// Test projection: project_shopping_list_item_collected updates read model
#[tokio::test]
async fn test_projection_shopping_list_item_collected() {
    // Setup: Create test database and executor
    let (pool, executor) = setup_test_db().await;

    // Create a shopping list
    let shopping_list_id = shopping::commands::generate_shopping_list(
        GenerateShoppingListCommand {
            user_id: "user-1".to_string(),
            meal_plan_id: "meal-plan-1".to_string(),
            week_start_date: "2025-10-20".to_string(),
            ingredients: vec![
                ("carrots".to_string(), 2.0, "lbs".to_string()),
                ("celery".to_string(), 1.0, "bunch".to_string()),
            ],
        },
        &executor,
    )
    .await
    .unwrap();

    // Process projection for ShoppingListGenerated
    shopping::shopping_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Get item IDs
    let item_id_0 = format!("{}-0", shopping_list_id);
    let item_id_1 = format!("{}-1", shopping_list_id);

    // Mark first item as collected
    mark_item_collected(
        MarkItemCollectedCommand {
            shopping_list_id: shopping_list_id.clone(),
            item_id: item_id_0.clone(),
            is_collected: true,
        },
        &executor,
    )
    .await
    .unwrap();

    // Process projection for ShoppingListItemCollected
    shopping::shopping_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify: Check is_collected status in read model
    let item_0: (bool,) =
        sqlx::query_as("SELECT is_collected FROM shopping_list_items WHERE id = ?")
            .bind(&item_id_0)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(item_0.0, "Item 0 should be collected");

    let item_1: (bool,) =
        sqlx::query_as("SELECT is_collected FROM shopping_list_items WHERE id = ?")
            .bind(&item_id_1)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!item_1.0, "Item 1 should not be collected");

    // Uncheck first item
    mark_item_collected(
        MarkItemCollectedCommand {
            shopping_list_id: shopping_list_id.clone(),
            item_id: item_id_0.clone(),
            is_collected: false,
        },
        &executor,
    )
    .await
    .unwrap();

    // Process projection
    shopping::shopping_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify: Item 0 should now be uncollected
    let item_0_uncollected: (bool,) =
        sqlx::query_as("SELECT is_collected FROM shopping_list_items WHERE id = ?")
            .bind(&item_id_0)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!item_0_uncollected.0, "Item 0 should be uncollected");
}

/// Test projection: project_shopping_list_reset unchecks all items
#[tokio::test]
async fn test_projection_shopping_list_reset() {
    // Setup: Create test database and executor
    let (pool, executor) = setup_test_db().await;

    // Create a shopping list with 3 items
    let shopping_list_id = shopping::commands::generate_shopping_list(
        GenerateShoppingListCommand {
            user_id: "user-1".to_string(),
            meal_plan_id: "meal-plan-1".to_string(),
            week_start_date: "2025-10-20".to_string(),
            ingredients: vec![
                ("potatoes".to_string(), 3.0, "lbs".to_string()),
                ("butter".to_string(), 0.5, "lb".to_string()),
                ("salt".to_string(), 1.0, "tsp".to_string()),
            ],
        },
        &executor,
    )
    .await
    .unwrap();

    // Process projection for ShoppingListGenerated
    shopping::shopping_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Mark all 3 items as collected
    for i in 0..3 {
        mark_item_collected(
            MarkItemCollectedCommand {
                shopping_list_id: shopping_list_id.clone(),
                item_id: format!("{}-{}", shopping_list_id, i),
                is_collected: true,
            },
            &executor,
        )
        .await
        .unwrap();
    }

    // Process projections for all ShoppingListItemCollected events
    shopping::shopping_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify: All items should be collected
    let collected_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM shopping_list_items WHERE shopping_list_id = ? AND is_collected = 1",
    )
    .bind(&shopping_list_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(collected_count.0, 3, "All 3 items should be collected");

    // Execute: Reset shopping list
    reset_shopping_list(
        ResetShoppingListCommand {
            shopping_list_id: shopping_list_id.clone(),
        },
        &executor,
    )
    .await
    .unwrap();

    // Process projection for ShoppingListReset
    shopping::shopping_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify: All items should now be uncollected
    let uncollected_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM shopping_list_items WHERE shopping_list_id = ? AND is_collected = 0",
    )
    .bind(&shopping_list_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        uncollected_count.0, 3,
        "All 3 items should be uncollected after reset"
    );
}
