use chrono::{Datelike, Duration, Utc};
use evento::prelude::{Migrate, Plan};
use shopping::{
    generate_shopping_list, shopping_projection, validate_week_date, GenerateShoppingListCommand,
    ShoppingListError,
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

    // Check categories using group_by_category (returns Vec, not HashMap)
    let categories = shopping_list.group_by_category();

    // Verify all 7 categories are present
    let category_names: Vec<&str> = categories.iter().map(|(name, _)| name.as_str()).collect();
    assert!(category_names.contains(&"Produce"));
    assert!(category_names.contains(&"Dairy"));
    assert!(category_names.contains(&"Meat"));
    assert!(category_names.contains(&"Pantry"));
    assert!(category_names.contains(&"Frozen"));
    assert!(category_names.contains(&"Bakery"));
    assert!(category_names.contains(&"Other"));
}

#[tokio::test]
async fn test_category_ordering_and_alphabetical_sorting() {
    let (pool, executor) = setup_test_db().await;

    // Prepare test ingredients with intentional disorder
    let ingredients = vec![
        // Bakery items (should be category 5)
        ("bread".to_string(), 1.0, "loaf".to_string()),
        ("bagels".to_string(), 6.0, "whole".to_string()),
        // Produce items (should be category 0 - first)
        ("zucchini".to_string(), 2.0, "whole".to_string()),
        ("tomato".to_string(), 3.0, "whole".to_string()),
        ("apple".to_string(), 5.0, "whole".to_string()),
        // Meat items (should be category 2)
        ("salmon".to_string(), 1.0, "lb".to_string()),
        ("chicken".to_string(), 2.0, "lb".to_string()),
        // Dairy items (should be category 1)
        ("yogurt".to_string(), 1.0, "cup".to_string()),
        ("milk".to_string(), 2.0, "cups".to_string()),
    ];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: "2025-10-13".to_string(),
        ingredients,
    };

    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    process_projections(&pool, &executor).await;

    let shopping_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Group by category (returns sorted Vec)
    let categories = shopping_list.group_by_category();

    // AC #7: Verify category order matches grocery store layout
    // Order: Produce(0), Dairy(1), Meat(2), Frozen(3), Pantry(4), Bakery(5), Other(6)
    assert_eq!(
        categories[0].0, "Produce",
        "First category should be Produce"
    );
    assert_eq!(categories[1].0, "Dairy", "Second category should be Dairy");
    assert_eq!(categories[2].0, "Meat", "Third category should be Meat");
    assert_eq!(
        categories[3].0, "Bakery",
        "Fourth category should be Bakery"
    );

    // AC #4: Verify items within each category are sorted alphabetically
    let produce_items = &categories[0].1;
    assert_eq!(produce_items.len(), 3);
    assert_eq!(produce_items[0].ingredient_name, "apple");
    assert_eq!(produce_items[1].ingredient_name, "tomato");
    assert_eq!(produce_items[2].ingredient_name, "zucchini");

    let dairy_items = &categories[1].1;
    assert_eq!(dairy_items.len(), 2);
    assert_eq!(dairy_items[0].ingredient_name, "milk");
    assert_eq!(dairy_items[1].ingredient_name, "yogurt");

    let meat_items = &categories[2].1;
    assert_eq!(meat_items.len(), 2);
    assert_eq!(meat_items[0].ingredient_name, "chicken");
    assert_eq!(meat_items[1].ingredient_name, "salmon");

    let bakery_items = &categories[3].1;
    assert_eq!(bakery_items.len(), 2);
    assert_eq!(bakery_items[0].ingredient_name, "bagels");
    assert_eq!(bakery_items[1].ingredient_name, "bread");
}

#[tokio::test]
async fn test_empty_categories_filtered() {
    let (pool, executor) = setup_test_db().await;

    // Prepare test ingredients from only Produce category
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

    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    process_projections(&pool, &executor).await;

    let shopping_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Group by category
    let categories = shopping_list.group_by_category();

    // AC #8: Empty categories should be hidden from view
    // Only Produce category should be present (all items are produce)
    assert_eq!(categories.len(), 1, "Only one category should be present");
    assert_eq!(categories[0].0, "Produce");
    assert_eq!(categories[0].1.len(), 3);

    // Verify no Dairy, Meat, Frozen, Pantry, Bakery, or Other categories
    let category_names: Vec<&str> = categories.iter().map(|(name, _)| name.as_str()).collect();
    assert!(!category_names.contains(&"Dairy"));
    assert!(!category_names.contains(&"Meat"));
    assert!(!category_names.contains(&"Frozen"));
    assert!(!category_names.contains(&"Pantry"));
    assert!(!category_names.contains(&"Bakery"));
    assert!(!category_names.contains(&"Other"));
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

// ==================== Story 4.3: Week Validation Tests ====================

/// Helper to get the Monday of the current week
fn get_current_week_monday() -> String {
    let today = Utc::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    monday.format("%Y-%m-%d").to_string()
}

/// Helper to get a future week's Monday (offset in weeks)
fn get_future_week_monday(weeks_ahead: i64) -> String {
    let today = Utc::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let future_monday = monday + Duration::weeks(weeks_ahead);
    future_monday.format("%Y-%m-%d").to_string()
}

/// Helper to get a past week's Monday (offset in weeks, negative)
fn get_past_week_monday(weeks_ago: i64) -> String {
    let today = Utc::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let past_monday = monday - Duration::weeks(weeks_ago);
    past_monday.format("%Y-%m-%d").to_string()
}

#[test]
fn test_validate_week_date_current_week() {
    // AC #3: Current week should be valid
    let current_week = get_current_week_monday();
    let result = validate_week_date(&current_week);
    assert!(result.is_ok(), "Current week should be valid");
}

#[test]
fn test_validate_week_date_next_week() {
    // AC #5: Future weeks up to +4 weeks should be valid
    let next_week = get_future_week_monday(1);
    let result = validate_week_date(&next_week);
    assert!(result.is_ok(), "Next week (+1) should be valid");
}

#[test]
fn test_validate_week_date_four_weeks_ahead() {
    // AC #5: +4 weeks is at the boundary, should be valid
    let four_weeks_ahead = get_future_week_monday(4);
    let result = validate_week_date(&four_weeks_ahead);
    assert!(
        result.is_ok(),
        "Four weeks ahead (+4) should be valid (boundary)"
    );
}

#[test]
fn test_validate_week_date_five_weeks_ahead_rejected() {
    // AC #5: +5 weeks exceeds the limit, should be rejected
    let five_weeks_ahead = get_future_week_monday(5);
    let result = validate_week_date(&five_weeks_ahead);
    assert!(matches!(
        result,
        Err(ShoppingListError::FutureWeekOutOfRangeError)
    ));
}

#[test]
fn test_validate_week_date_past_week_rejected() {
    // AC #7: Past weeks should be rejected
    let last_week = get_past_week_monday(1);
    let result = validate_week_date(&last_week);
    assert!(matches!(
        result,
        Err(ShoppingListError::PastWeekNotAccessibleError)
    ));
}

#[test]
fn test_validate_week_date_two_weeks_ago_rejected() {
    // AC #7: Past weeks should be rejected
    let two_weeks_ago = get_past_week_monday(2);
    let result = validate_week_date(&two_weeks_ago);
    assert!(matches!(
        result,
        Err(ShoppingListError::PastWeekNotAccessibleError)
    ));
}

#[test]
fn test_validate_week_date_invalid_format() {
    // Invalid date format should be rejected
    let result = validate_week_date("invalid-date");
    assert!(matches!(
        result,
        Err(ShoppingListError::InvalidWeekError(_))
    ));
}

#[test]
fn test_validate_week_date_not_monday() {
    // Non-Monday dates should be rejected
    // 2025-10-22 is a Tuesday
    let result = validate_week_date("2025-10-22");
    assert!(
        matches!(result, Err(ShoppingListError::InvalidWeekError(_))),
        "Tuesday should be rejected, only Mondays allowed"
    );
}

#[test]
fn test_validate_week_date_invalid_date_format() {
    // Invalid ISO 8601 format should be rejected
    let result = validate_week_date("2025-13-99");
    assert!(matches!(
        result,
        Err(ShoppingListError::InvalidWeekError(_))
    ));
}

#[tokio::test]
async fn test_get_shopping_list_by_week_with_validation() {
    let (pool, executor) = setup_test_db().await;

    // Generate a shopping list for current week
    let current_week = get_current_week_monday();
    let ingredients = vec![("tomato".to_string(), 2.0, "whole".to_string())];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: current_week.clone(),
        ingredients,
    };

    generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    process_projections(&pool, &executor).await;

    // Query with valid current week - should succeed
    let result =
        shopping::read_model::get_shopping_list_by_week("user-1", &current_week, &pool).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());

    // Query with past week - should fail validation
    let past_week = get_past_week_monday(1);
    let result = shopping::read_model::get_shopping_list_by_week("user-1", &past_week, &pool).await;
    assert!(matches!(
        result,
        Err(ShoppingListError::PastWeekNotAccessibleError)
    ));

    // Query with invalid date format - should fail validation
    let result =
        shopping::read_model::get_shopping_list_by_week("user-1", "invalid-date", &pool).await;
    assert!(matches!(
        result,
        Err(ShoppingListError::InvalidWeekError(_))
    ));

    // Query with non-Monday - should fail validation
    let result =
        shopping::read_model::get_shopping_list_by_week("user-1", "2025-10-22", &pool).await;
    assert!(matches!(
        result,
        Err(ShoppingListError::InvalidWeekError(_))
    ));
}

#[tokio::test]
async fn test_get_shopping_list_by_week_returns_none_for_nonexistent_week() {
    let (pool, _executor) = setup_test_db().await;

    // Query a valid future week that has no shopping list yet
    let next_week = get_future_week_monday(1);
    let result = shopping::read_model::get_shopping_list_by_week("user-1", &next_week, &pool)
        .await
        .expect("Query should succeed even if no list exists");

    // Should return None, not an error (week is valid but no list exists yet)
    assert!(
        result.is_none(),
        "Should return None when no shopping list exists for valid week"
    );
}
