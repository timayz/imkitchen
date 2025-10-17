/// Integration tests for meal planning feature (Story 3.1)
///
/// These tests verify the full evento event → read model projection flow
/// and HTTP route behavior with actual database operations.
use chrono::Utc;
use evento::prelude::*;
use meal_planning::{events::MealPlanGenerated, read_model::MealPlanQueries, MealPlanAggregate};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Helper: Create in-memory test database with migrations
async fn create_test_db() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("Failed to create test database");

    // Initialize evento event store schema
    let mut conn = pool.acquire().await.expect("Failed to acquire connection");
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut conn, &Plan::apply_all())
        .await
        .expect("Failed to run evento migrations");
    drop(conn);

    // Run application migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Helper: Create test user in database
async fn create_test_user(
    pool: &SqlitePool,
    user_id: &str,
    email: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, tier, created_at)
        VALUES (?1, ?2, 'hash', 'free', ?3)
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

/// Helper: Create test recipes for meal planning
async fn create_test_recipes(
    pool: &SqlitePool,
    user_id: &str,
    count: usize,
) -> Result<Vec<String>, sqlx::Error> {
    let mut recipe_ids = Vec::new();

    for i in 1..=count {
        let recipe_id = format!("recipe_{}", i);
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO recipes (
                id, user_id, title, ingredients, instructions,
                prep_time_min, cook_time_min, serving_size,
                is_favorite, is_shared, complexity, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind(&recipe_id)
        .bind(user_id)
        .bind(format!("Recipe {}", i))
        .bind(r#"[{"name":"ingredient1","amount":"1 cup"}]"#)
        .bind(r#"[{"step_number":1,"instruction":"Cook it"}]"#)
        .bind(15 + (i as i32 % 10))
        .bind(30)
        .bind(4)
        .bind(true) // is_favorite
        .bind(false)
        .bind("simple")
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        recipe_ids.push(recipe_id);
    }

    Ok(recipe_ids)
}

#[tokio::test]
async fn test_meal_plan_generated_event_projects_to_read_model() {
    // Setup: Create test database and evento executor
    let pool = create_test_db().await;
    let user_id = "user_test_1";

    create_test_user(&pool, user_id, "test@example.com")
        .await
        .expect("Failed to create test user");

    // Create evento executor with SQLite backend
    let evento_executor: evento::Sqlite = pool.clone().into();

    // Create test recipes (meal_assignments table has FK to recipes table)
    create_test_recipes(&pool, user_id, 5)
        .await
        .expect("Failed to create test recipes");

    // Act: Create MealPlanGenerated event via evento
    let start_date = "2025-10-20".to_string();
    let meal_assignments = vec![
        meal_planning::events::MealAssignment {
            date: "2025-10-20".to_string(),
            meal_type: "breakfast".to_string(),
            recipe_id: "recipe_1".to_string(),
            prep_required: false,
        },
        meal_planning::events::MealAssignment {
            date: "2025-10-20".to_string(),
            meal_type: "lunch".to_string(),
            recipe_id: "recipe_2".to_string(),
            prep_required: false,
        },
    ];

    let event_data = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: start_date.clone(),
        meal_assignments: meal_assignments.clone(),
        rotation_state_json: r#"{"cycle_number":1,"used_recipe_ids":["recipe_1","recipe_2"]}"#
            .to_string(),
        generated_at: Utc::now().to_rfc3339(),
    };

    let _aggregator_id = evento::create::<MealPlanAggregate>()
        .data(&event_data)
        .expect("Failed to encode event data")
        .metadata(&true)
        .expect("Failed to encode metadata")
        .commit(&evento_executor)
        .await
        .expect("Failed to commit event");

    // Use unsafe_oneshot to synchronously process the event projection
    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&evento_executor)
        .await
        .expect("Failed to process meal plan projection");

    // Assert: Verify read model was projected
    let meal_plan = MealPlanQueries::get_active_meal_plan(user_id, &pool)
        .await
        .expect("Failed to query meal plan");

    assert!(
        meal_plan.is_some(),
        "Meal plan should be projected to read model"
    );

    let meal_plan = meal_plan.unwrap();
    assert_eq!(meal_plan.user_id, user_id);
    assert_eq!(meal_plan.start_date, start_date);
    assert_eq!(meal_plan.status, "active");

    // Verify meal assignments were projected
    let assignments = MealPlanQueries::get_meal_assignments(&meal_plan.id, &pool)
        .await
        .expect("Failed to query meal assignments");

    assert_eq!(assignments.len(), 2, "Should have 2 meal assignments");
    assert_eq!(assignments[0].meal_type, "breakfast");
    assert_eq!(assignments[1].meal_type, "lunch");
}

#[tokio::test]
async fn test_insufficient_recipes_returns_error() {
    // This test verifies AC-10: If insufficient recipes (<7 favorites), display helpful error
    // Since we're testing the algorithm layer, we'll test the MealPlanningAlgorithm directly

    use meal_planning::algorithm::{MealPlanningAlgorithm, RecipeForPlanning, UserConstraints};
    use meal_planning::rotation::RotationState;
    use meal_planning::MealPlanningError;

    let favorites = vec![
        RecipeForPlanning {
            id: "1".to_string(),
            title: "Recipe 1".to_string(),
            ingredients_count: 5,
            instructions_count: 4,
            prep_time_min: Some(15),
            cook_time_min: Some(30),
            advance_prep_hours: None,
            complexity: None,
        },
        RecipeForPlanning {
            id: "2".to_string(),
            title: "Recipe 2".to_string(),
            ingredients_count: 8,
            instructions_count: 6,
            prep_time_min: Some(20),
            cook_time_min: Some(40),
            advance_prep_hours: None,
            complexity: None,
        },
    ];

    let constraints = UserConstraints::default();
    let rotation_state = RotationState::new();

    let result =
        MealPlanningAlgorithm::generate("2025-10-20", favorites, constraints, rotation_state, Some(42));

    assert!(result.is_err());
    match result {
        Err(MealPlanningError::InsufficientRecipes { minimum, current }) => {
            assert_eq!(minimum, 7);
            assert_eq!(current, 2);
        }
        _ => panic!("Expected InsufficientRecipes error"),
    }
}

#[tokio::test]
async fn test_rotation_state_persists_across_generations() {
    // This test verifies that rotation state is properly maintained across multiple meal plan generations
    let pool = create_test_db().await;
    let user_id = "user_rotation_test";

    create_test_user(&pool, user_id, "rotation@example.com")
        .await
        .expect("Failed to create test user");

    let evento_executor: evento::Sqlite = pool.clone().into();

    // First generation with 7 recipes
    let event_data_1 = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: "2025-10-20".to_string(),
        meal_assignments: vec![],
        rotation_state_json:
            r#"{"cycle_number":1,"used_recipe_ids":["r1","r2","r3","r4","r5","r6","r7"]}"#
                .to_string(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event_data_1)
        .expect("Failed to encode event data")
        .metadata(&true)
        .expect("Failed to encode metadata")
        .commit(&evento_executor)
        .await
        .expect("Failed to commit first event");

    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&evento_executor)
        .await
        .expect("Failed to process first meal plan projection");

    // Verify first meal plan rotation state
    let meal_plan_1 = MealPlanQueries::get_active_meal_plan(user_id, &pool)
        .await
        .expect("Failed to query first meal plan")
        .expect("First meal plan should exist");

    assert!(meal_plan_1.rotation_state.contains("r1"));
    assert!(meal_plan_1.rotation_state.contains("r7"));

    // Second generation should archive the first and create new one
    let event_data_2 = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: "2025-10-27".to_string(),
        meal_assignments: vec![],
        rotation_state_json: r#"{"cycle_number":2,"used_recipe_ids":[]}"#.to_string(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event_data_2)
        .expect("Failed to encode event data")
        .metadata(&true)
        .expect("Failed to encode metadata")
        .commit(&evento_executor)
        .await
        .expect("Failed to commit second event");

    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&evento_executor)
        .await
        .expect("Failed to process second meal plan projection");

    // Verify second meal plan is active and first is archived
    let active_plan = MealPlanQueries::get_active_meal_plan(user_id, &pool)
        .await
        .expect("Failed to query active meal plan")
        .expect("Active meal plan should exist");

    assert_eq!(active_plan.start_date, "2025-10-27");
    assert!(active_plan.rotation_state.contains(r#""cycle_number":2"#));
}

#[tokio::test]
async fn test_multiple_meal_assignments_projected_correctly() {
    // This test verifies that all 21 meal assignments (7 days × 3 meals) are correctly projected
    let pool = create_test_db().await;
    let user_id = "user_full_week";

    create_test_user(&pool, user_id, "fullweek@example.com")
        .await
        .expect("Failed to create test user");

    let evento_executor: evento::Sqlite = pool.clone().into();

    // Create test recipes
    create_test_recipes(&pool, user_id, 10)
        .await
        .expect("Failed to create test recipes");

    // Create 21 meal assignments (7 days × 3 meals)
    let mut meal_assignments = Vec::new();
    for day in 0..7 {
        let date = format!("2025-10-{}", 20 + day);
        for meal_type in ["breakfast", "lunch", "dinner"] {
            meal_assignments.push(meal_planning::events::MealAssignment {
                date: date.clone(),
                meal_type: meal_type.to_string(),
                recipe_id: format!("recipe_{}", (day * 3 + meal_assignments.len() % 3) % 10 + 1),
                prep_required: false,
            });
        }
    }

    let event_data = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: "2025-10-20".to_string(),
        meal_assignments: meal_assignments.clone(),
        rotation_state_json: r#"{"cycle_number":1,"used_recipe_ids":[]}"#.to_string(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event_data)
        .expect("Failed to encode event data")
        .metadata(&true)
        .expect("Failed to encode metadata")
        .commit(&evento_executor)
        .await
        .expect("Failed to commit event");

    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&evento_executor)
        .await
        .expect("Failed to process meal plan projection");

    // Verify all assignments projected
    let meal_plan = MealPlanQueries::get_active_meal_plan(user_id, &pool)
        .await
        .expect("Failed to query meal plan")
        .expect("Meal plan should exist");

    let assignments = MealPlanQueries::get_meal_assignments(&meal_plan.id, &pool)
        .await
        .expect("Failed to query assignments");

    assert_eq!(
        assignments.len(),
        21,
        "Should have 21 meal assignments (7 days × 3 meals)"
    );

    // Verify we have assignments for all meal types
    let breakfast_count = assignments
        .iter()
        .filter(|a| a.meal_type == "breakfast")
        .count();
    let lunch_count = assignments
        .iter()
        .filter(|a| a.meal_type == "lunch")
        .count();
    let dinner_count = assignments
        .iter()
        .filter(|a| a.meal_type == "dinner")
        .count();

    assert_eq!(breakfast_count, 7);
    assert_eq!(lunch_count, 7);
    assert_eq!(dinner_count, 7);
}
