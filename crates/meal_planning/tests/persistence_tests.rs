/// Unit tests for Story 3.11: Meal Plan Persistence and Activation
///
/// These tests verify that:
/// - Meal plans are persisted to database upon generation (AC #1)
/// - Only one active meal plan exists per user at a time (AC #2, #10)
/// - Active meal plans can be queried and loaded (AC #3, #4, #8)
/// - Regeneration archives old plan and creates new active plan (AC #5)
/// - Meal plan ID remains stable during replacements (AC #9)
///
/// Test Strategy: Use unsafe_oneshot for synchronous event processing
use chrono::Utc;
use evento::prelude::{Migrate, Plan};
use meal_planning::{
    aggregate::MealPlanAggregate,
    constraints::CourseType,
    events::{MealAssignment, MealPlanGenerated},
    read_model::{meal_plan_projection, MealPlanQueries},
};
use sqlx::sqlite::SqlitePoolOptions;

/// Setup in-memory test database with evento migrations and read model tables
async fn setup_test_db() -> (evento::Sqlite, sqlx::SqlitePool) {
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

    // Run read model migrations
    // Migration 01: users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT UNIQUE NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Migration 01: recipes table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS recipes (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            title TEXT NOT NULL,
            is_favorite INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Migration 02: meal_plans table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS meal_plans (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            start_date TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('active', 'archived')),
            rotation_state TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Migration 03: Unique constraint for single active meal plan (Story 3.11 AC #2, #10)
    sqlx::query(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_meal_plans_unique_active
        ON meal_plans(user_id)
        WHERE status = 'active'
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Migration 02: meal_assignments table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS meal_assignments (
            id TEXT PRIMARY KEY NOT NULL,
            meal_plan_id TEXT NOT NULL,
            date TEXT NOT NULL,
            course_type TEXT NOT NULL CHECK(course_type IN ('appetizer', 'main_course', 'dessert')),
            recipe_id TEXT NOT NULL,
            prep_required INTEGER NOT NULL DEFAULT 0,
            assignment_reasoning TEXT,
            FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE,
            FOREIGN KEY (recipe_id) REFERENCES recipes(id)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Migration (rotation): recipe_rotation_state table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS recipe_rotation_state (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            cycle_number INTEGER NOT NULL,
            recipe_id TEXT NOT NULL,
            used_at TEXT NOT NULL,
            UNIQUE(user_id, cycle_number, recipe_id)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    (executor, pool)
}

/// Helper: Create test user
async fn create_test_user(user_id: &str, pool: &sqlx::SqlitePool) {
    sqlx::query("INSERT INTO users (id, email, created_at) VALUES (?1, ?2, ?3)")
        .bind(user_id)
        .bind(format!("{}@test.com", user_id))
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .unwrap();
}

/// Helper: Create test recipe
async fn create_test_recipe(recipe_id: &str, user_id: &str, pool: &sqlx::SqlitePool) {
    sqlx::query(
        "INSERT INTO recipes (id, user_id, title, is_favorite, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind(format!("Recipe {}", recipe_id))
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .unwrap();
}

/// Helper: Create sample meal assignments (7 days × 3 meals = 21 assignments)
fn create_sample_assignments(start_date: &str) -> Vec<MealAssignment> {
    let mut assignments = Vec::new();
    let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d").unwrap();

    for day_offset in 0..7 {
        let date = start + chrono::Duration::days(day_offset);
        let date_str = date.format("%Y-%m-%d").to_string();

        for course_type in [CourseType::Appetizer, CourseType::MainCourse, CourseType::Dessert] {
            assignments.push(MealAssignment {
                date: date_str.clone(),
                course_type: course_type.as_str().to_string(),
                recipe_id: format!("recipe_{}", assignments.len() + 1),
                prep_required: false,
                assignment_reasoning: None,
            });
        }
    }

    assignments
}

/// Test: Meal plan persisted to database immediately upon generation (AC #1)
#[tokio::test]
async fn test_meal_plan_persisted_on_generation() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user1";
    let start_date = "2025-10-20";

    create_test_user(user_id, &pool).await;

    // Create recipes
    for i in 1..=21 {
        create_test_recipe(&format!("recipe_{}", i), user_id, &pool).await;
    }

    // Emit MealPlanGenerated event
    let assignments = create_sample_assignments(start_date);
    let event_data = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: start_date.to_string(),
        meal_assignments: assignments.clone(),
        rotation_state_json: format!(
            r#"{{"cycle_number":1,"cycle_started_at":"{}","used_recipe_ids":[],"total_favorite_count":21}}"#,
            Utc::now().to_rfc3339()
        ),
        generated_at: Utc::now().to_rfc3339(),
    };

    let meal_plan_id = evento::create::<MealPlanAggregate>()
        .data(&event_data)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Register projection subscription with unsafe_oneshot for sync processing
    meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC #1: Verify meal plan exists in database
    let meal_plan = MealPlanQueries::get_meal_plan_by_id(&meal_plan_id, &pool)
        .await
        .unwrap()
        .expect("Meal plan should be persisted");

    assert_eq!(meal_plan.user_id, user_id);
    assert_eq!(meal_plan.start_date, start_date);
    assert_eq!(meal_plan.status, "active");

    // Verify all 21 assignments persisted
    let assignments_in_db = MealPlanQueries::get_meal_assignments(&meal_plan_id, &pool)
        .await
        .unwrap();

    assert_eq!(
        assignments_in_db.len(),
        21,
        "All 21 meal assignments should be persisted"
    );
}

/// Test: Only one active meal plan per user (AC #2, #10)
#[tokio::test]
async fn test_single_active_meal_plan_enforced() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user2";
    let start_date_1 = "2025-10-20";
    let start_date_2 = "2025-10-27";

    create_test_user(user_id, &pool).await;

    // Create recipes
    for i in 1..=21 {
        create_test_recipe(&format!("recipe_{}", i), user_id, &pool).await;
    }

    // Generate first meal plan
    let assignments_1 = create_sample_assignments(start_date_1);
    let event_1 = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: start_date_1.to_string(),
        meal_assignments: assignments_1,
        rotation_state_json: format!(
            r#"{{"cycle_number":1,"cycle_started_at":"{}","used_recipe_ids":[],"total_favorite_count":21}}"#,
            Utc::now().to_rfc3339()
        ),
        generated_at: Utc::now().to_rfc3339(),
    };

    let meal_plan_id_1 = evento::create::<MealPlanAggregate>()
        .data(&event_1)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Process first event
    meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify first plan is active
    let plan_1 = MealPlanQueries::get_meal_plan_by_id(&meal_plan_id_1, &pool)
        .await
        .unwrap()
        .expect("First plan should exist");
    assert_eq!(plan_1.status, "active");

    // Generate second meal plan (should archive first)
    let assignments_2 = create_sample_assignments(start_date_2);
    let event_2 = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: start_date_2.to_string(),
        meal_assignments: assignments_2,
        rotation_state_json: format!(
            r#"{{"cycle_number":1,"cycle_started_at":"{}","used_recipe_ids":[],"total_favorite_count":21}}"#,
            Utc::now().to_rfc3339()
        ),
        generated_at: Utc::now().to_rfc3339(),
    };

    let meal_plan_id_2 = evento::create::<MealPlanAggregate>()
        .data(&event_2)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Process second event
    meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC #2, #10: Verify only one active plan exists
    let plan_1_after = MealPlanQueries::get_meal_plan_by_id(&meal_plan_id_1, &pool)
        .await
        .unwrap()
        .expect("First plan should still exist");
    assert_eq!(
        plan_1_after.status, "archived",
        "First plan should be archived"
    );

    let plan_2 = MealPlanQueries::get_meal_plan_by_id(&meal_plan_id_2, &pool)
        .await
        .unwrap()
        .expect("Second plan should exist");
    assert_eq!(plan_2.status, "active", "Second plan should be active");

    // Verify query returns only the active plan
    let active_plan = MealPlanQueries::get_active_meal_plan(user_id, &pool)
        .await
        .unwrap()
        .expect("Active plan should be queryable");
    assert_eq!(active_plan.id, meal_plan_id_2);
}

/// Test: Active meal plan automatically loaded and displayed (AC #3, #4, #8)
#[tokio::test]
async fn test_active_meal_plan_query() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user3";
    let start_date = "2025-10-20";

    create_test_user(user_id, &pool).await;

    // Create recipes
    for i in 1..=21 {
        create_test_recipe(&format!("recipe_{}", i), user_id, &pool).await;
    }

    // Generate meal plan
    let assignments = create_sample_assignments(start_date);
    let event = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: start_date.to_string(),
        meal_assignments: assignments,
        rotation_state_json: format!(
            r#"{{"cycle_number":1,"cycle_started_at":"{}","used_recipe_ids":[],"total_favorite_count":21}}"#,
            Utc::now().to_rfc3339()
        ),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Process event
    meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC #3, #8: Query active meal plan (simulates dashboard/calendar loading)
    let active_plan = MealPlanQueries::get_active_meal_plan(user_id, &pool)
        .await
        .unwrap()
        .expect("Active plan should be queryable");

    assert_eq!(active_plan.user_id, user_id);
    assert_eq!(active_plan.status, "active");

    // AC #3: Query with assignments (simulates calendar view)
    let plan_with_assignments =
        MealPlanQueries::get_active_meal_plan_with_assignments(user_id, &pool)
            .await
            .unwrap()
            .expect("Active plan with assignments should be queryable");

    assert_eq!(plan_with_assignments.meal_plan.id, active_plan.id);
    assert_eq!(
        plan_with_assignments.assignments.len(),
        21,
        "All assignments should be loaded"
    );
}

/// Test: No active meal plan returns None (AC #8 - error handling)
#[tokio::test]
async fn test_no_active_meal_plan_returns_none() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user4";

    create_test_user(user_id, &pool).await;

    // Register projection
    meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // AC #8: Query when no plan exists (simulates new user visiting dashboard)
    let active_plan = MealPlanQueries::get_active_meal_plan(user_id, &pool)
        .await
        .unwrap();

    assert!(
        active_plan.is_none(),
        "Should return None when no active plan exists"
    );

    let plan_with_assignments =
        MealPlanQueries::get_active_meal_plan_with_assignments(user_id, &pool)
            .await
            .unwrap();

    assert!(
        plan_with_assignments.is_none(),
        "Should return None when no active plan with assignments exists"
    );
}

/// Test: Cross-session persistence (AC #4)
#[tokio::test]
async fn test_cross_session_persistence() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user5";
    let start_date = "2025-10-20";

    create_test_user(user_id, &pool).await;

    // Create recipes
    for i in 1..=21 {
        create_test_recipe(&format!("recipe_{}", i), user_id, &pool).await;
    }

    // Session 1: Generate meal plan
    let assignments = create_sample_assignments(start_date);
    let event = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: start_date.to_string(),
        meal_assignments: assignments,
        rotation_state_json: format!(
            r#"{{"cycle_number":1,"cycle_started_at":"{}","used_recipe_ids":[],"total_favorite_count":21}}"#,
            Utc::now().to_rfc3339()
        ),
        generated_at: Utc::now().to_rfc3339(),
    };

    let meal_plan_id = evento::create::<MealPlanAggregate>()
        .data(&event)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Simulate session end (drop pool connection)
    drop(executor);

    // Session 2: New executor, same pool (simulates browser reopen)
    let executor2: evento::Sqlite = pool.clone().into();

    // AC #4: Meal plan should still be accessible
    let active_plan = MealPlanQueries::get_active_meal_plan(user_id, &pool)
        .await
        .unwrap()
        .expect("Meal plan should persist across sessions");

    assert_eq!(active_plan.id, meal_plan_id);
    assert_eq!(active_plan.status, "active");

    drop(executor2);
}

/// Test: Idempotency - reprocessing event doesn't duplicate data (AC #1)
#[tokio::test]
async fn test_projection_idempotency() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user6";
    let start_date = "2025-10-20";

    create_test_user(user_id, &pool).await;

    // Create recipes
    for i in 1..=21 {
        create_test_recipe(&format!("recipe_{}", i), user_id, &pool).await;
    }

    // Generate meal plan
    let assignments = create_sample_assignments(start_date);
    let event = MealPlanGenerated {
        user_id: user_id.to_string(),
        start_date: start_date.to_string(),
        meal_assignments: assignments,
        rotation_state_json: format!(
            r#"{{"cycle_number":1,"cycle_started_at":"{}","used_recipe_ids":[],"total_favorite_count":21}}"#,
            Utc::now().to_rfc3339()
        ),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Process event first time
    meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    let count_1: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM meal_plans")
        .fetch_one(&pool)
        .await
        .unwrap();

    // Process event second time (simulates event replay)
    meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    let count_2: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM meal_plans")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(
        count_1, count_2,
        "Reprocessing event should not duplicate meal plans (idempotency)"
    );
}
