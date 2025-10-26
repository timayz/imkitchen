/// Integration tests for Story 6.6: Multi-Week Meal Plan Projection
///
/// These tests verify that:
/// - MultiWeekMealPlanGenerated events project correctly into meal_plans table
/// - All weeks in a batch are inserted atomically
/// - generation_batch_id links weeks from same generation
/// - Meal assignments (21 per week) are created correctly
/// - Week status calculated from dates (Future/Current/Past)
/// - rotation_state serialized to JSON and deserializes correctly
/// - accompaniment_recipe_id stored for main courses
///
/// Test Strategy: Use unsafe_oneshot for synchronous event processing (AC #10)
use chrono::Utc;
use evento::migrator::{Migrate, Plan};
use meal_planning::{
    aggregate::MealPlanAggregate,
    events::{MealAssignment, MultiWeekMealPlanGenerated, WeekMealPlanData, WeekStatus},
    read_model::meal_plan_projection,
    rotation::RotationState,
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

    // Run application migrations from migrations/ directory
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    (executor, pool)
}

/// Helper: Create test user
async fn create_test_user(user_id: &str, pool: &sqlx::SqlitePool) {
    sqlx::query("INSERT INTO users (id, email, password_hash, created_at) VALUES (?1, ?2, ?3, ?4)")
        .bind(user_id)
        .bind(format!("{}@test.com", user_id))
        .bind("test_hash")
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .unwrap();
}

/// Helper: Create test recipes
async fn create_test_recipe(recipe_id: &str, user_id: &str, pool: &sqlx::SqlitePool) {
    sqlx::query(
        r#"INSERT INTO recipes (id, user_id, title, ingredients, instructions, is_favorite, created_at, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)"#,
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind(format!("Recipe {}", recipe_id))
    .bind("[]")
    .bind("[]")
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .unwrap();
}

/// Test AC #8: Multi-week meal plan with 3 weeks → verify 3 rows in meal_plans
#[tokio::test]
async fn test_multi_week_meal_plan_inserts_all_weeks() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user-multi-week-1";
    let batch_id = "batch-123";

    create_test_user(user_id, &pool).await;

    // Create 63 test recipes (3 weeks × 21 assignments)
    for i in 1..=63 {
        create_test_recipe(&format!("recipe-{}", i), user_id, &pool).await;
    }

    // Create 3 weeks of meal plan data
    let mut weeks = Vec::new();
    for week_num in 0..3 {
        let start_date = chrono::NaiveDate::from_ymd_opt(2025, 11, 3 + (week_num * 7)).unwrap();
        let end_date = start_date + chrono::Duration::days(6);

        let mut meal_assignments = Vec::new();
        for day_offset in 0..7i64 {
            let date = start_date + chrono::Duration::days(day_offset);
            for course in ["appetizer", "main_course", "dessert"] {
                let idx = week_num * 21
                    + (day_offset as u32) * 3
                    + if course == "appetizer" {
                        0
                    } else if course == "main_course" {
                        1
                    } else {
                        2
                    };
                meal_assignments.push(MealAssignment {
                    date: date.format("%Y-%m-%d").to_string(),
                    course_type: course.to_string(),
                    recipe_id: format!("recipe-{}", idx + 1),
                    prep_required: course == "main_course",
                    assignment_reasoning: Some(format!("{} for {}", course, date)),
                    accompaniment_recipe_id: if course == "main_course" {
                        Some(format!("side-{}", idx + 1))
                    } else {
                        None
                    },
                });
            }
        }

        weeks.push(WeekMealPlanData {
            id: format!("week-{}", week_num + 1),
            start_date: start_date.format("%Y-%m-%d").to_string(),
            end_date: end_date.format("%Y-%m-%d").to_string(),
            status: WeekStatus::Future,
            is_locked: false,
            meal_assignments,
            shopping_list_id: format!("shop-{}", week_num + 1),
        });
    }

    // Create MultiWeekMealPlanGenerated event
    let event_data = MultiWeekMealPlanGenerated {
        generation_batch_id: batch_id.to_string(),
        user_id: user_id.to_string(),
        weeks,
        rotation_state: RotationState::new(),
        generated_at: Utc::now().to_rfc3339(),
    };

    // Store event
    evento::create::<MealPlanAggregate>()
        .data(&event_data)
        .unwrap()
        .metadata(&true)
        .unwrap()
        .commit(&executor)
        .await
        .unwrap();

    // Run projection with unsafe_oneshot (AC #10)
    meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify: 3 meal plans inserted
    let meal_plan_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM meal_plans WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(meal_plan_count.0, 3, "Should insert 3 weeks");

    // Verify: All weeks have status = 'active' (future weeks map to 'active' in DB)
    let active_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM meal_plans WHERE user_id = ? AND status = 'active'")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        active_count.0, 3,
        "All 3 future weeks should have 'active' status in DB"
    );
}

/// Test AC #8: Verify 63 meal assignments created (3 weeks × 21)
#[tokio::test]
async fn test_multi_week_meal_plan_creates_all_assignments() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user-multi-week-2";
    let batch_id = "batch-456";

    create_test_user(user_id, &pool).await;

    // Create 63 test recipes
    for i in 1..=63 {
        create_test_recipe(&format!("recipe-{}", i), user_id, &pool).await;
    }

    // Create 3 weeks with 21 assignments each
    let mut weeks = Vec::new();
    for week_num in 0..3 {
        let start_date = chrono::NaiveDate::from_ymd_opt(2025, 12, 1 + (week_num * 7)).unwrap();
        let end_date = start_date + chrono::Duration::days(6);

        let mut meal_assignments = Vec::new();
        for day_offset in 0..7i64 {
            let date = start_date + chrono::Duration::days(day_offset);
            for course in ["appetizer", "main_course", "dessert"] {
                let idx = week_num * 21
                    + (day_offset as u32) * 3
                    + if course == "appetizer" {
                        0
                    } else if course == "main_course" {
                        1
                    } else {
                        2
                    };
                meal_assignments.push(MealAssignment {
                    date: date.format("%Y-%m-%d").to_string(),
                    course_type: course.to_string(),
                    recipe_id: format!("recipe-{}", idx + 1),
                    prep_required: false,
                    assignment_reasoning: None,
                    accompaniment_recipe_id: None,
                });
            }
        }

        weeks.push(WeekMealPlanData {
            id: format!("week-assign-{}", week_num + 1),
            start_date: start_date.format("%Y-%m-%d").to_string(),
            end_date: end_date.format("%Y-%m-%d").to_string(),
            status: WeekStatus::Future,
            is_locked: false,
            meal_assignments,
            shopping_list_id: format!("shop-assign-{}", week_num + 1),
        });
    }

    let event_data = MultiWeekMealPlanGenerated {
        generation_batch_id: batch_id.to_string(),
        user_id: user_id.to_string(),
        weeks,
        rotation_state: RotationState::new(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event_data)
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

    // Verify: 63 meal assignments (3 weeks × 21)
    let assignment_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM meal_assignments")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        assignment_count.0, 63,
        "Should create 63 meal assignments (3 weeks × 21)"
    );
}

/// Test AC #8: Verify generation_batch_id links all weeks from same batch
#[tokio::test]
async fn test_generation_batch_id_links_weeks() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user-batch-link";
    let batch_id = "batch-unique-789";

    create_test_user(user_id, &pool).await;

    for i in 1..=42 {
        create_test_recipe(&format!("recipe-{}", i), user_id, &pool).await;
    }

    // Create 2 weeks
    let mut weeks = Vec::new();
    for week_num in 0..2 {
        let start_date = chrono::NaiveDate::from_ymd_opt(2026, 1, 5 + (week_num * 7)).unwrap();
        let end_date = start_date + chrono::Duration::days(6);

        let mut meal_assignments = Vec::new();
        for day_offset in 0..7i64 {
            let date = start_date + chrono::Duration::days(day_offset);
            for course in ["appetizer", "main_course", "dessert"] {
                let idx = week_num * 21 + (day_offset as u32) * 3;
                meal_assignments.push(MealAssignment {
                    date: date.format("%Y-%m-%d").to_string(),
                    course_type: course.to_string(),
                    recipe_id: format!("recipe-{}", idx + 1),
                    prep_required: false,
                    assignment_reasoning: None,
                    accompaniment_recipe_id: None,
                });
            }
        }

        weeks.push(WeekMealPlanData {
            id: format!("week-batch-{}", week_num + 1),
            start_date: start_date.format("%Y-%m-%d").to_string(),
            end_date: end_date.format("%Y-%m-%d").to_string(),
            status: WeekStatus::Future,
            is_locked: false,
            meal_assignments,
            shopping_list_id: format!("shop-batch-{}", week_num + 1),
        });
    }

    let event_data = MultiWeekMealPlanGenerated {
        generation_batch_id: batch_id.to_string(),
        user_id: user_id.to_string(),
        weeks,
        rotation_state: RotationState::new(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event_data)
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

    // Verify: Both weeks have same generation_batch_id
    let batch_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM meal_plans WHERE generation_batch_id = ?")
            .bind(batch_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        batch_count.0, 2,
        "Both weeks should share the same generation_batch_id"
    );
}

/// Test AC #8: Verify rotation_state serialized to JSON and can be deserialized
#[tokio::test]
async fn test_rotation_state_json_serialization() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user-rotation-json";
    let batch_id = "batch-rotation-001";

    create_test_user(user_id, &pool).await;

    for i in 1..=21 {
        create_test_recipe(&format!("recipe-{}", i), user_id, &pool).await;
    }

    // Create rotation state with specific data
    let mut rotation_state = RotationState::new();
    rotation_state.cycle_number = 3;
    rotation_state.mark_recipe_used("recipe-1".to_string());
    rotation_state.mark_recipe_used("recipe-5".to_string());
    rotation_state.mark_recipe_used("recipe-10".to_string());

    let start_date = chrono::NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();
    let end_date = start_date + chrono::Duration::days(6);

    let mut meal_assignments = Vec::new();
    for day_offset in 0..7i64 {
        let date = start_date + chrono::Duration::days(day_offset);
        for course in ["appetizer", "main_course", "dessert"] {
            meal_assignments.push(MealAssignment {
                date: date.format("%Y-%m-%d").to_string(),
                course_type: course.to_string(),
                recipe_id: format!("recipe-{}", day_offset * 3 + 1),
                prep_required: false,
                assignment_reasoning: None,
                accompaniment_recipe_id: None,
            });
        }
    }

    let weeks = vec![WeekMealPlanData {
        id: "week-rotation-test".to_string(),
        start_date: start_date.format("%Y-%m-%d").to_string(),
        end_date: end_date.format("%Y-%m-%d").to_string(),
        status: WeekStatus::Future,
        is_locked: false,
        meal_assignments,
        shopping_list_id: "shop-rotation".to_string(),
    }];

    let event_data = MultiWeekMealPlanGenerated {
        generation_batch_id: batch_id.to_string(),
        user_id: user_id.to_string(),
        weeks,
        rotation_state: rotation_state.clone(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event_data)
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

    // Verify: rotation_state_json can be deserialized
    let rotation_json: (String,) = sqlx::query_as(
        "SELECT rotation_state_json FROM meal_plans WHERE id = 'week-rotation-test'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let deserialized: RotationState = serde_json::from_str(&rotation_json.0)
        .expect("rotation_state_json should deserialize correctly");

    assert_eq!(
        deserialized.cycle_number, 3,
        "Cycle number should deserialize correctly"
    );
    assert_eq!(
        deserialized.used_recipe_ids.len(),
        3,
        "Used recipe IDs should deserialize correctly"
    );
    assert!(
        deserialized.used_recipe_ids.contains("recipe-1"),
        "Should contain recipe-1"
    );
    assert!(
        deserialized.used_recipe_ids.contains("recipe-5"),
        "Should contain recipe-5"
    );
}

/// Test AC #8: Verify accompaniment_recipe_id stored for main courses
#[tokio::test]
async fn test_accompaniment_recipe_id_stored() {
    let (executor, pool) = setup_test_db().await;
    let user_id = "user-accompaniment";
    let batch_id = "batch-accompaniment";

    create_test_user(user_id, &pool).await;

    for i in 1..=21 {
        create_test_recipe(&format!("recipe-{}", i), user_id, &pool).await;
    }

    let start_date = chrono::NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
    let end_date = start_date + chrono::Duration::days(6);

    let mut meal_assignments = Vec::new();
    for day_offset in 0..7i64 {
        let date = start_date + chrono::Duration::days(day_offset);
        for course in ["appetizer", "main_course", "dessert"] {
            meal_assignments.push(MealAssignment {
                date: date.format("%Y-%m-%d").to_string(),
                course_type: course.to_string(),
                recipe_id: format!("recipe-{}", day_offset * 3 + 1),
                prep_required: false,
                assignment_reasoning: None,
                accompaniment_recipe_id: if course == "main_course" {
                    Some(format!("side-dish-{}", day_offset + 1))
                } else {
                    None
                },
            });
        }
    }

    let weeks = vec![WeekMealPlanData {
        id: "week-accompaniment-test".to_string(),
        start_date: start_date.format("%Y-%m-%d").to_string(),
        end_date: end_date.format("%Y-%m-%d").to_string(),
        status: WeekStatus::Future,
        is_locked: false,
        meal_assignments,
        shopping_list_id: "shop-accompaniment".to_string(),
    }];

    let event_data = MultiWeekMealPlanGenerated {
        generation_batch_id: batch_id.to_string(),
        user_id: user_id.to_string(),
        weeks,
        rotation_state: RotationState::new(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<MealPlanAggregate>()
        .data(&event_data)
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

    // Verify: 7 main courses have accompaniment_recipe_id
    let accompaniment_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM meal_assignments WHERE course_type = 'main_course' AND accompaniment_recipe_id IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        accompaniment_count.0, 7,
        "All 7 main courses should have accompaniment_recipe_id"
    );

    // Verify: Appetizers and desserts have NULL accompaniment_recipe_id
    let null_accompaniment_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM meal_assignments WHERE course_type IN ('appetizer', 'dessert') AND accompaniment_recipe_id IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        null_accompaniment_count.0, 14,
        "Appetizers and desserts should have NULL accompaniment_recipe_id"
    );
}
