use chrono::{Datelike, Duration, Utc};
use evento::prelude::{Migrate, Plan};
use shopping::{
    generate_shopping_list, recalculate_shopping_list_on_meal_replacement, shopping_projection,
    GenerateShoppingListCommand, RecalculateShoppingListCommand, ShoppingListError,
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

/// Helper to process projections after event creation
async fn process_projections(pool: &SqlitePool, executor: &evento::Sqlite) {
    shopping_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();
}

/// Helper to get the Monday of the current week
fn get_current_week_monday() -> String {
    let today = Utc::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    monday.format("%Y-%m-%d").to_string()
}

// ==================== Story 4.4: Shopping List Recalculation Tests ====================

#[tokio::test]
async fn test_recalculate_shopping_list_replace_meal_basic() {
    // AC #1, #2, #3, #4: Replacing meal slot triggers recalculation with old ingredients subtracted, new ingredients added, quantities re-aggregated
    let (pool, executor) = setup_test_db().await;

    let current_week = get_current_week_monday();

    // Step 1: Generate initial shopping list with 2 recipes
    // Recipe 1: chicken 1lb, tomato 2 whole
    // Recipe 2: chicken 1lb, onion 1 whole
    // Expected: chicken 2lbs (907.18g), tomato 2 whole (2 item), onion 1 whole (1 item)
    let initial_ingredients = vec![
        ("chicken".to_string(), 1.0, "lb".to_string()),
        ("tomato".to_string(), 2.0, "whole".to_string()),
        ("chicken".to_string(), 1.0, "lb".to_string()),
        ("onion".to_string(), 1.0, "whole".to_string()),
    ];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: current_week.clone(),
        ingredients: initial_ingredients,
    };

    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    process_projections(&pool, &executor).await;

    // Verify initial shopping list
    let shopping_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    assert_eq!(shopping_list.items.len(), 3);

    let chicken = shopping_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "chicken")
        .expect("Chicken not found");
    assert!((chicken.quantity - 907.18).abs() < 0.1); // 2 lbs ≈ 907.18 grams

    // Step 2: Replace Recipe 1 with a new recipe
    // Old recipe (Recipe 1): chicken 1lb, tomato 2 whole
    // New recipe (Recipe 3): beef 2lbs, tomato 1 whole
    // Expected after recalculation:
    //   - chicken 1lb (from Recipe 2 only, 453.59g)
    //   - tomato 1 whole (removed 2 from Recipe 1, added 1 from Recipe 3, net: 2 - 2 + 1 = 1)
    //   - onion 1 whole (unchanged from Recipe 2)
    //   - beef 2lbs (new from Recipe 3, 907.18g)
    let old_recipe_ingredients = vec![
        ("chicken".to_string(), 1.0, "lb".to_string()),
        ("tomato".to_string(), 2.0, "whole".to_string()),
    ];

    let new_recipe_ingredients = vec![
        ("beef".to_string(), 2.0, "lbs".to_string()),
        ("tomato".to_string(), 1.0, "whole".to_string()),
    ];

    let recalc_command = RecalculateShoppingListCommand {
        shopping_list_id: shopping_list_id.clone(),
        old_recipe_ingredients,
        new_recipe_ingredients,
    };

    recalculate_shopping_list_on_meal_replacement(recalc_command, &executor)
        .await
        .expect("Failed to recalculate shopping list");

    process_projections(&pool, &executor).await;

    // Step 3: Verify recalculated shopping list
    let updated_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    assert_eq!(updated_list.items.len(), 4);

    // Verify chicken quantity reduced from 2lbs to 1lb
    let chicken = updated_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "chicken")
        .expect("Chicken not found");
    assert!((chicken.quantity - 453.59).abs() < 0.1); // 1 lb ≈ 453.59 grams

    // Verify tomato quantity reduced from 2 to 1
    let tomato = updated_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "tomato")
        .expect("Tomato not found");
    assert_eq!(tomato.quantity, 1.0);
    assert_eq!(tomato.unit, "item");

    // Verify onion unchanged
    let onion = updated_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "onion")
        .expect("Onion not found");
    assert_eq!(onion.quantity, 1.0);

    // Verify beef added
    let beef = updated_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "beef")
        .expect("Beef not found");
    assert!((beef.quantity - 907.18).abs() < 0.1); // 2 lbs ≈ 907.18 grams
}

#[tokio::test]
async fn test_recalculate_shopping_list_remove_only_recipe_for_ingredient() {
    // Edge case: Removed recipe was the only recipe requiring an ingredient → remove ingredient from list
    let (pool, executor) = setup_test_db().await;

    let current_week = get_current_week_monday();

    // Initial list: Recipe 1 (chicken 1lb, tomato 2 whole) + Recipe 2 (onion 1 whole)
    let initial_ingredients = vec![
        ("chicken".to_string(), 1.0, "lb".to_string()),
        ("tomato".to_string(), 2.0, "whole".to_string()),
        ("onion".to_string(), 1.0, "whole".to_string()),
    ];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: current_week.clone(),
        ingredients: initial_ingredients,
    };

    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    process_projections(&pool, &executor).await;

    // Replace Recipe 1 with a recipe that doesn't use chicken or tomato
    // Old: chicken 1lb, tomato 2 whole
    // New: beef 1lb
    let old_recipe_ingredients = vec![
        ("chicken".to_string(), 1.0, "lb".to_string()),
        ("tomato".to_string(), 2.0, "whole".to_string()),
    ];

    let new_recipe_ingredients = vec![("beef".to_string(), 1.0, "lb".to_string())];

    let recalc_command = RecalculateShoppingListCommand {
        shopping_list_id: shopping_list_id.clone(),
        old_recipe_ingredients,
        new_recipe_ingredients,
    };

    recalculate_shopping_list_on_meal_replacement(recalc_command, &executor)
        .await
        .expect("Failed to recalculate shopping list");

    process_projections(&pool, &executor).await;

    // Verify: chicken and tomato removed (were only in Recipe 1), onion and beef remain
    let updated_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    assert_eq!(updated_list.items.len(), 2);

    assert!(updated_list
        .items
        .iter()
        .all(|item| item.ingredient_name != "chicken"));
    assert!(updated_list
        .items
        .iter()
        .all(|item| item.ingredient_name != "tomato"));
    assert!(updated_list
        .items
        .iter()
        .any(|item| item.ingredient_name == "onion"));
    assert!(updated_list
        .items
        .iter()
        .any(|item| item.ingredient_name == "beef"));
}

#[tokio::test]
async fn test_recalculate_shopping_list_add_ingredient_at_zero_quantity() {
    // Edge case: New recipe adds ingredient already at zero quantity → restore to list with new quantity
    let (pool, executor) = setup_test_db().await;

    let current_week = get_current_week_monday();

    // Initial list: Recipe 1 (chicken 1lb) + Recipe 2 (chicken 1lb)
    let initial_ingredients = vec![
        ("chicken".to_string(), 1.0, "lb".to_string()),
        ("chicken".to_string(), 1.0, "lb".to_string()),
    ];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: current_week.clone(),
        ingredients: initial_ingredients,
    };

    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    process_projections(&pool, &executor).await;

    // Step 1: Replace Recipe 1 to remove all chicken (make quantity 0)
    // Old: chicken 2lbs (aggregated)
    // Replace Recipe 1: remove chicken 1lb, add beef 1lb
    // Expected: chicken 1lb remaining (from Recipe 2)
    let old_recipe_1 = vec![("chicken".to_string(), 1.0, "lb".to_string())];
    let new_recipe_1 = vec![("beef".to_string(), 1.0, "lb".to_string())];

    recalculate_shopping_list_on_meal_replacement(
        RecalculateShoppingListCommand {
            shopping_list_id: shopping_list_id.clone(),
            old_recipe_ingredients: old_recipe_1,
            new_recipe_ingredients: new_recipe_1,
        },
        &executor,
    )
    .await
    .expect("Failed to recalculate shopping list");

    process_projections(&pool, &executor).await;

    // Step 2: Replace Recipe 2 to remove last chicken
    // Old: chicken 1lb
    // New: tomato 2 whole
    // Expected: chicken removed (quantity 0), only beef and tomato remain
    let old_recipe_2 = vec![("chicken".to_string(), 1.0, "lb".to_string())];
    let new_recipe_2 = vec![("tomato".to_string(), 2.0, "whole".to_string())];

    recalculate_shopping_list_on_meal_replacement(
        RecalculateShoppingListCommand {
            shopping_list_id: shopping_list_id.clone(),
            old_recipe_ingredients: old_recipe_2,
            new_recipe_ingredients: new_recipe_2,
        },
        &executor,
    )
    .await
    .expect("Failed to recalculate shopping list");

    process_projections(&pool, &executor).await;

    let list_after_removal = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Chicken should be removed (zero quantity)
    assert!(list_after_removal
        .items
        .iter()
        .all(|item| item.ingredient_name != "chicken"));

    // Step 3: Now add chicken back with a new recipe
    // Old: tomato 2 whole
    // New: chicken 3lbs, tomato 1 whole
    // Expected: chicken restored with 3lbs (1360.77g)
    let old_recipe_3 = vec![("tomato".to_string(), 2.0, "whole".to_string())];
    let new_recipe_3 = vec![
        ("chicken".to_string(), 3.0, "lbs".to_string()),
        ("tomato".to_string(), 1.0, "whole".to_string()),
    ];

    recalculate_shopping_list_on_meal_replacement(
        RecalculateShoppingListCommand {
            shopping_list_id: shopping_list_id.clone(),
            old_recipe_ingredients: old_recipe_3,
            new_recipe_ingredients: new_recipe_3,
        },
        &executor,
    )
    .await
    .expect("Failed to recalculate shopping list");

    process_projections(&pool, &executor).await;

    let final_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    // Chicken should be restored with 3lbs
    let chicken = final_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "chicken")
        .expect("Chicken should be restored");
    assert!((chicken.quantity - 1360.77).abs() < 0.1); // 3 lbs ≈ 1360.77 grams
}

#[tokio::test]
async fn test_recalculate_shopping_list_preserve_collected_status() {
    // AC #6 (Task 2): Preserve item checkoff state during recalculation (don't reset checked items)
    let (pool, executor) = setup_test_db().await;

    let current_week = get_current_week_monday();

    // Initial list: chicken 1lb, tomato 2 whole, onion 1 whole
    let initial_ingredients = vec![
        ("chicken".to_string(), 1.0, "lb".to_string()),
        ("tomato".to_string(), 2.0, "whole".to_string()),
        ("onion".to_string(), 1.0, "whole".to_string()),
    ];

    let command = GenerateShoppingListCommand {
        user_id: "user-1".to_string(),
        meal_plan_id: "meal-plan-1".to_string(),
        week_start_date: current_week.clone(),
        ingredients: initial_ingredients,
    };

    let shopping_list_id = generate_shopping_list(command, &executor)
        .await
        .expect("Failed to generate shopping list");

    process_projections(&pool, &executor).await;

    // Manually mark onion as collected in database
    sqlx::query("UPDATE shopping_list_items SET is_collected = 1 WHERE ingredient_name = 'onion'")
        .execute(&pool)
        .await
        .expect("Failed to mark onion as collected");

    // Recalculate: replace chicken recipe
    let old_recipe = vec![("chicken".to_string(), 1.0, "lb".to_string())];
    let new_recipe = vec![("beef".to_string(), 1.0, "lb".to_string())];

    recalculate_shopping_list_on_meal_replacement(
        RecalculateShoppingListCommand {
            shopping_list_id: shopping_list_id.clone(),
            old_recipe_ingredients: old_recipe,
            new_recipe_ingredients: new_recipe,
        },
        &executor,
    )
    .await
    .expect("Failed to recalculate shopping list");

    process_projections(&pool, &executor).await;

    // Verify onion is still marked as collected after recalculation
    let updated_list = shopping::read_model::get_shopping_list(&shopping_list_id, &pool)
        .await
        .expect("Failed to query shopping list")
        .expect("Shopping list not found");

    let onion = updated_list
        .items
        .iter()
        .find(|item| item.ingredient_name == "onion")
        .expect("Onion should still be present");
    assert!(
        onion.is_collected,
        "Onion should still be marked as collected after recalculation"
    );
}
