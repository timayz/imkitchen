use chrono::Utc;
use evento::prelude::{Migrate, Plan};
use meal_planning::algorithm::{RecipeForPlanning, UserConstraints};
use meal_planning::events::MealPlanGenerated;
use meal_planning::read_model::MealPlanQueries;
use meal_planning::{MealPlanAggregate, MealPlanningAlgorithm};
use sqlx::sqlite::SqlitePoolOptions;

async fn setup_test_db() -> (evento::Sqlite, sqlx::SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();

    // Run evento migrations
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    // Run read model migrations
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS meal_plans (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            start_date TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('active', 'archived')),
            rotation_state TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS meal_assignments (
            id TEXT PRIMARY KEY NOT NULL,
            meal_plan_id TEXT NOT NULL,
            date TEXT NOT NULL,
            meal_type TEXT NOT NULL CHECK(meal_type IN ('breakfast', 'lunch', 'dinner')),
            recipe_id TEXT NOT NULL,
            prep_required INTEGER NOT NULL DEFAULT 0,
            assignment_reasoning TEXT,
            FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    (executor, pool)
}

fn create_test_recipe(id: &str, complexity_score: (usize, usize)) -> RecipeForPlanning {
    RecipeForPlanning {
        id: id.to_string(),
        title: format!("Recipe {}", id),
        ingredients_count: complexity_score.0,
        instructions_count: complexity_score.1,
        prep_time_min: Some(15),
        cook_time_min: Some(30),
        advance_prep_hours: None,
        complexity: None,
    }
}

#[tokio::test]
async fn test_reasoning_persisted_to_database() {
    // Setup
    let (executor, pool) = setup_test_db().await;

    // Create executor for committing events
    let commit_executor: evento::Sqlite = pool.clone().into();

    // Create test recipes
    let favorites = vec![
        create_test_recipe("r1", (6, 4)),   // Simple
        create_test_recipe("r2", (8, 6)),   // Simple
        create_test_recipe("r3", (10, 8)),  // Moderate
        create_test_recipe("r4", (12, 10)), // Moderate
        create_test_recipe("r5", (15, 12)), // Moderate
        create_test_recipe("r6", (18, 14)), // Moderate
        create_test_recipe("r7", (20, 16)), // Moderate
    ];

    // Generate meal plan
    let user_constraints = UserConstraints::default();
    let rotation_state = meal_planning::rotation::RotationState::new();

    let (assignments, rotation_state) = MealPlanningAlgorithm::generate(
        "2025-10-20", // Monday
        favorites,
        user_constraints,
        rotation_state,
        Some(42), // Fixed seed
    )
    .expect("Generation should succeed");

    // Verify assignments have reasoning
    for assignment in &assignments {
        assert!(
            assignment.assignment_reasoning.is_some(),
            "Assignment should have reasoning"
        );
        let reasoning = assignment.assignment_reasoning.as_ref().unwrap();
        assert!(!reasoning.is_empty(), "Reasoning should not be empty");
    }

    // Emit MealPlanGenerated event
    let event_data = MealPlanGenerated {
        user_id: "test_user".to_string(),
        start_date: "2025-10-20".to_string(),
        meal_assignments: assignments.clone(),
        rotation_state_json: rotation_state.to_json().unwrap(),
        generated_at: Utc::now().to_rfc3339(),
    };

    let meal_plan_id = evento::create::<MealPlanAggregate>()
        .data(&event_data)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&commit_executor)
        .await
        .unwrap();

    // Process projections synchronously (use unsafe_oneshot as per user's instruction)
    let subscription = meal_planning::read_model::meal_plan_projection(pool.clone());
    subscription.unsafe_oneshot(&executor).await.unwrap();

    // Query read model and verify reasoning persisted
    let stored_assignments = MealPlanQueries::get_meal_assignments(&meal_plan_id, &pool)
        .await
        .expect("Query should succeed");

    assert_eq!(stored_assignments.len(), 21, "Should have 21 assignments");

    // Verify each assignment has reasoning
    for stored in &stored_assignments {
        assert!(
            stored.assignment_reasoning.is_some(),
            "Stored assignment should have reasoning for date={} meal_type={}",
            stored.date,
            stored.meal_type
        );

        let reasoning = stored.assignment_reasoning.as_ref().unwrap();
        assert!(
            !reasoning.is_empty(),
            "Stored reasoning should not be empty"
        );
        assert!(
            reasoning.len() >= 20,
            "Reasoning should be meaningful (>=20 chars): {}",
            reasoning
        );
        assert!(
            reasoning.len() <= 120,
            "Reasoning should be concise (<=120 chars): {}",
            reasoning
        );
    }

    // Verify specific reasoning examples
    let tuesday_dinner = stored_assignments
        .iter()
        .find(|a| a.date == "2025-10-21" && a.meal_type == "dinner")
        .expect("Should find Tuesday dinner");

    let reasoning = tuesday_dinner.assignment_reasoning.as_ref().unwrap();
    assert!(
        reasoning.contains("Tuesday"),
        "Reasoning should mention day: {}",
        reasoning
    );
}

#[tokio::test]
async fn test_reasoning_query_returns_with_assignments() {
    // Setup
    let (executor, pool) = setup_test_db().await;

    // Create executor for committing events
    let commit_executor: evento::Sqlite = pool.clone().into();

    // Create and generate meal plan
    let favorites = (1..=10)
        .map(|i| create_test_recipe(&format!("r{}", i), (5 + i, 4 + i)))
        .collect();

    let (assignments, rotation_state) = MealPlanningAlgorithm::generate(
        "2025-10-20",
        favorites,
        UserConstraints::default(),
        meal_planning::rotation::RotationState::new(),
        Some(99),
    )
    .unwrap();

    // Emit event
    let event_data = MealPlanGenerated {
        user_id: "test_user_2".to_string(),
        start_date: "2025-10-20".to_string(),
        meal_assignments: assignments,
        rotation_state_json: rotation_state.to_json().unwrap(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event_data)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&commit_executor)
        .await
        .unwrap();

    // Process projections synchronously (use unsafe_oneshot as per user's instruction)
    let subscription = meal_planning::read_model::meal_plan_projection(pool.clone());
    subscription.unsafe_oneshot(&executor).await.unwrap();

    // Query with assignments
    let result = MealPlanQueries::get_active_meal_plan_with_assignments("test_user_2", &pool)
        .await
        .expect("Query should succeed");

    assert!(result.is_some(), "Should find meal plan");
    let meal_plan_with_assignments = result.unwrap();

    assert_eq!(
        meal_plan_with_assignments.assignments.len(),
        21,
        "Should have 21 assignments"
    );

    // Verify all assignments have reasoning
    for assignment in &meal_plan_with_assignments.assignments {
        assert!(
            assignment.assignment_reasoning.is_some(),
            "Assignment reasoning should be present"
        );
    }
}
