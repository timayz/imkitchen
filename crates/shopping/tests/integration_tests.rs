use evento::prelude::{Migrate, Plan};
use shopping::{generate_shopping_list, shopping_projection, GenerateShoppingListCommand};
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
            generated_at TEXT NOT NULL
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

/// Helper to process projections after event creation
async fn process_projections(pool: &SqlitePool, executor: &evento::Sqlite) {
    shopping_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_generate_shopping_list_basic() {
    let (pool, executor) = setup_test_db().await;

    // Prepare test ingredients (simple case)
    let ingredients = vec![
        ("tomato".to_string(), 2.0, "whole".to_string()),
        ("onion".to_string(), 1.0, "whole".to_string()),
        ("garlic".to_string(), 3.0, "cloves".to_string()),
    ];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: "2025-10-13".to_string(),
        ingredients,
    };

    // Execute command
    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    // Process projection
    process_projections(&pool, &executor).await;

    // Query shopping list from read model
    let shopping_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Assertions
    assert_eq!(shopping_list.header.user_id, "user-1");
    assert_eq!(shopping_list.header.meal_plan_id, "meal-plan-1");
    assert_eq!(shopping_list.header.week_start_date, "2025-10-13");
    assert_eq!(shopping_list.items.len(), 3);

    // Check that items are categorized
    assert!(shopping_list
        .items
        .iter()
        .all(|item| !item.category.is_empty()));
}

#[tokio::test]
async fn test_generate_shopping_list_with_aggregation() {
    let (pool, executor) = setup_test_db().await;

    // Prepare test ingredients with duplicates requiring aggregation
    let ingredients = vec![
        ("tomato".to_string(), 2.0, "whole".to_string()),
        ("tomato".to_string(), 3.0, "whole".to_string()), // Should aggregate to 5 whole
        ("milk".to_string(), 1.0, "cup".to_string()),
        ("milk".to_string(), 2.0, "cups".to_string()), // Should aggregate to 3 cups -> 720ml
        ("onion".to_string(), 1.0, "whole".to_string()),
    ];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: "2025-10-13".to_string(),
        ingredients,
    };

    // Execute command
    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    // Process projection
    process_projections(&pool, &executor).await;

    // Query shopping list from read model
    let shopping_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Assertions
    assert_eq!(shopping_list.items.len(), 3); // 3 unique ingredients after aggregation

    // Find tomato (should be aggregated to 5 items)
    let tomato = shopping_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "tomato")
        .expect("Tomato not found");
    assert_eq!(tomato.quantity, 5.0);
    assert_eq!(tomato.unit, "item");

    // Find milk (should be aggregated to 720ml)
    let milk = shopping_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "milk")
        .expect("Milk not found");
    assert_eq!(milk.quantity, 720.0);
    assert_eq!(milk.unit, "ml");

    // Find onion (should be 1 item)
    let onion = shopping_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "onion")
        .expect("Onion not found");
    assert_eq!(onion.quantity, 1.0);
    assert_eq!(onion.unit, "item");
}

#[tokio::test]
async fn test_generate_shopping_list_with_unit_conversion() {
    let (pool, executor) = setup_test_db().await;

    // Prepare test ingredients requiring unit conversion
    let ingredients = vec![
        ("flour".to_string(), 2.0, "cups".to_string()), // 480ml
        ("flour".to_string(), 100.0, "ml".to_string()), // Should aggregate to 580ml
        ("chicken".to_string(), 1.0, "lb".to_string()), // 453.59g
        ("chicken".to_string(), 200.0, "g".to_string()), // Should aggregate to 653.59g
        ("olive oil".to_string(), 3.0, "tbsp".to_string()), // 45ml
    ];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: "2025-10-13".to_string(),
        ingredients,
    };

    // Execute command
    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    // Process projection
    process_projections(&pool, &executor).await;

    // Query shopping list from read model
    let shopping_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Assertions
    assert_eq!(shopping_list.items.len(), 3);

    // Find flour (should be aggregated to 580ml)
    let flour = shopping_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "flour")
        .expect("Flour not found");
    assert_eq!(flour.quantity, 580.0);
    assert_eq!(flour.unit, "ml");

    // Find chicken (should be aggregated to 653.59g)
    let chicken = shopping_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "chicken")
        .expect("Chicken not found");
    assert!((chicken.quantity - 653.59).abs() < 0.01); // floating point comparison
    assert_eq!(chicken.unit, "g");

    // Find olive oil (should be 45ml)
    let olive_oil = shopping_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "olive oil")
        .expect("Olive oil not found");
    assert_eq!(olive_oil.quantity, 45.0);
    assert_eq!(olive_oil.unit, "ml");
}

#[tokio::test]
async fn test_generate_shopping_list_categorization() {
    let (pool, executor) = setup_test_db().await;

    // Prepare test ingredients from different categories
    let ingredients = vec![
        ("tomato".to_string(), 2.0, "whole".to_string()), // Produce
        ("milk".to_string(), 1.0, "cup".to_string()),     // Dairy
        ("chicken".to_string(), 1.0, "lb".to_string()),   // Meat
        ("flour".to_string(), 2.0, "cups".to_string()),   // Pantry
        ("frozen peas".to_string(), 1.0, "cup".to_string()), // Frozen
        ("bread".to_string(), 1.0, "loaf".to_string()),   // Bakery
        ("weird ingredient".to_string(), 1.0, "unit".to_string()), // Other
    ];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: "2025-10-13".to_string(),
        ingredients,
    };

    // Execute command
    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    // Process projection
    process_projections(&pool, &executor).await;

    // Query shopping list from read model
    let shopping_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Assertions
    assert_eq!(shopping_list.items.len(), 7);

    // Check categories
    let categories = shopping_list.group_by_category();
    assert!(categories.contains_key("Produce"));
    assert!(categories.contains_key("Dairy"));
    assert!(categories.contains_key("Meat"));
    assert!(categories.contains_key("Pantry"));
    assert!(categories.contains_key("Frozen"));
    assert!(categories.contains_key("Bakery"));
    assert!(categories.contains_key("Other"));
}

#[tokio::test]
async fn test_query_shopping_list_by_week() {
    let (pool, executor) = setup_test_db().await;

    let ingredients = vec![("tomato".to_string(), 2.0, "whole".to_string())];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: "2025-10-13".to_string(),
        ingredients,
    };

    // Execute command
    generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    // Process projection
    process_projections(&pool, &executor).await;

    // Query by week
    let shopping_list =
        shopping::read_model::get_shopping_list_by_week("user-1", "2025-10-13", &pool)
            .await
            .expect("Failed to query shopping list by week")
            .expect("Shopping list not found");

    // Assertions
    assert_eq!(shopping_list.header.user_id, "user-1");
    assert_eq!(shopping_list.header.week_start_date, "2025-10-13");
    assert_eq!(shopping_list.items.len(), 1);
}

#[tokio::test]
async fn test_generate_shopping_list_empty_ingredients() {
    let (pool, executor) = setup_test_db().await;

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: "2025-10-13".to_string(),
        ingredients: vec![],
    };

    // Execute command - should succeed with empty list
    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    // Process projection
    process_projections(&pool, &executor).await;

    // Query shopping list
    let shopping_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Should have no items
    assert_eq!(shopping_list.items.len(), 0);
}

#[tokio::test]
async fn test_generate_shopping_list_large_dataset() {
    let (pool, executor) = setup_test_db().await;

    // Create a large dataset (14 recipes * 10 ingredients each = 140 ingredients)
    let mut ingredients = Vec::new();
    let ingredient_names = [
        "tomato", "onion", "garlic", "chicken", "beef", "rice", "flour", "milk", "eggs", "cheese",
    ];

    for i in 0..14 {
        for (j, name) in ingredient_names.iter().enumerate() {
            ingredients.push((
                format!("{}-{}", name, i),
                (j + 1) as f32,
                "whole".to_string(),
            ));
        }
    }

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: "2025-10-13".to_string(),
        ingredients: ingredients.clone(),
    };

    // Measure performance
    let start = std::time::Instant::now();
    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");
    let elapsed = start.elapsed();

    // Process projection
    process_projections(&pool, &executor).await;

    // Query shopping list
    let shopping_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Assertions
    assert_eq!(shopping_list.items.len(), 140);

    // Performance assertion: should complete in < 2 seconds
    assert!(
        elapsed.as_secs() < 2,
        "Shopping list generation took too long: {:?}",
        elapsed
    );

    println!("✓ Generated shopping list with 140 items in {:?}", elapsed);
}
