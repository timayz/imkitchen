/// Integration tests for Regenerate All Future Weeks Route (Story 8.4)
///
/// These tests verify the POST /plan/regenerate-all-future route behavior:
/// - AC-1: Route POST /plan/regenerate-all-future created
/// - AC-2: Requires confirmation parameter (prevent accidental regeneration)
/// - AC-3: Handler identifies current week (locked) and preserves it
/// - AC-4: Handler regenerates all future weeks (status == "future")
/// - AC-5: Handler resets rotation state but preserves current week's main courses
/// - AC-6: Handler commits AllFutureWeeksRegenerated event to evento
/// - AC-7: Handler regenerates shopping lists for all future weeks (via projection)
/// - AC-8: Returns count of regenerated weeks + first future week data
/// - AC-9: Integration test - POST with confirmation regenerates all future weeks
/// - AC-10: Integration test - POST without confirmation returns 400
use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode},
    Router,
};
use chrono::{Datelike, Duration, Utc};
use evento::migrator::{Migrate, Plan};
use http_body_util::BodyExt;
use meal_planning::{
    events::{MealAssignment, MultiWeekMealPlanGenerated, WeekMealPlanData, WeekStatus},
    rotation::RotationState,
    MealPlanAggregate,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tower::ServiceExt;

// Import the route handler and AppState
use imkitchen::middleware::auth::Auth;
use imkitchen::routes::{regenerate_all_future_weeks, AppState};

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

/// Helper: Create test user using evento
async fn create_test_user(
    pool: &SqlitePool,
    user_id: &str,
    email: &str,
) -> Result<(), anyhow::Error> {
    let now = Utc::now().to_rfc3339();

    let executor: evento::Sqlite = pool.clone().into();

    let event_data = user::events::UserCreated {
        email: email.to_string(),
        password_hash: "test_hash".to_string(),
        created_at: now.clone(),
    };

    let generated_id = evento::create::<user::UserAggregate>()
        .data(&event_data)
        .expect("Failed to encode UserCreated event data")
        .metadata(&true)
        .expect("Failed to encode event metadata")
        .commit(&executor)
        .await
        .expect("Failed to commit UserCreated event");

    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .expect("Failed to process user projection");

    sqlx::query("UPDATE users SET id = ?1 WHERE id = ?2")
        .bind(user_id)
        .bind(&generated_id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE event SET aggregator_id = ?1 WHERE aggregator_id = ?2")
        .bind(user_id)
        .bind(&generated_id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE snapshot SET id = ?1 WHERE id = ?2")
        .bind(user_id)
        .bind(&generated_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Helper: Create test recipes (all three types)
async fn create_test_recipes(
    pool: &SqlitePool,
    user_id: &str,
    count_per_type: usize,
) -> Result<Vec<String>, sqlx::Error> {
    let mut recipe_ids = Vec::new();
    let now = Utc::now().to_rfc3339();

    let recipe_types = vec!["appetizer", "main_course", "dessert"];

    for recipe_type in recipe_types {
        for i in 1..=count_per_type {
            let recipe_id = format!("{}_{}", recipe_type, i);

            sqlx::query(
                r#"
                INSERT INTO recipes (
                    id, user_id, title, complexity, prep_time_min, cook_time_min,
                    advance_prep_hours, serving_size, is_shared, is_favorite,
                    created_at, updated_at, recipe_type, cuisine, dietary_tags,
                    ingredients, instructions
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
                "#,
            )
            .bind(&recipe_id)
            .bind(user_id)
            .bind(format!("Test {} Recipe {}", recipe_type, i))
            .bind("moderate")
            .bind(20) // prep_time_min
            .bind(30) // cook_time_min
            .bind(0) // advance_prep_hours
            .bind(4) // serving_size
            .bind(0) // is_shared (INTEGER)
            .bind(1) // is_favorite (INTEGER)
            .bind(&now)
            .bind(&now)
            .bind(recipe_type) // recipe_type
            .bind("italian")
            .bind("[]")
            .bind("[]") // ingredients
            .bind("[]") // instructions
            .execute(pool)
            .await?;

            recipe_ids.push(recipe_id);
        }
    }

    Ok(recipe_ids)
}

/// Helper: Create existing multi-week meal plan with current + future weeks
async fn create_existing_meal_plan(
    pool: &SqlitePool,
    user_id: &str,
    recipe_ids: &[String],
) -> Result<String, anyhow::Error> {
    let executor: evento::Sqlite = pool.clone().into();

    // Create multi-week meal plan (5 weeks)
    let generation_batch_id = uuid::Uuid::new_v4().to_string();
    let today = Utc::now().date_naive();
    let start_of_week = today - Duration::days(today.weekday().number_from_monday() as i64);

    let mut weeks = Vec::new();

    for week_offset in 0..5 {
        let week_start = start_of_week + Duration::weeks(week_offset);
        let week_end = week_start + Duration::days(6);
        let week_id = format!("week_{}", week_offset + 1);

        // Week 0 is current (locked), weeks 1-4 are future (unlocked)
        let status = if week_offset == 0 {
            WeekStatus::Current
        } else {
            WeekStatus::Future
        };
        let is_locked = week_offset == 0;

        // Create meal assignments (21 per week: 7 days × 3 meals)
        let mut meal_assignments = Vec::new();
        for day_offset in 0..7 {
            let date = week_start + Duration::days(day_offset);
            for course_idx in 0..3 {
                let recipe_id =
                    &recipe_ids[(week_offset as usize * 21 + day_offset as usize * 3 + course_idx)
                        % recipe_ids.len()];

                meal_assignments.push(MealAssignment {
                    date: date.format("%Y-%m-%d").to_string(),
                    course_type: match course_idx {
                        0 => "appetizer".to_string(),
                        1 => "main_course".to_string(),
                        _ => "dessert".to_string(),
                    },
                    recipe_id: recipe_id.clone(),
                    prep_required: false,
                    assignment_reasoning: Some("Test assignment".to_string()),
                    accompaniment_recipe_id: None,
                });
            }
        }

        weeks.push(WeekMealPlanData {
            id: week_id.clone(),
            start_date: week_start.format("%Y-%m-%d").to_string(),
            end_date: week_end.format("%Y-%m-%d").to_string(),
            status,
            is_locked,
            meal_assignments,
            shopping_list_id: format!("shopping_{}", week_id),
        });
    }

    // Emit MultiWeekMealPlanGenerated event
    let event_data = MultiWeekMealPlanGenerated {
        generation_batch_id: generation_batch_id.clone(),
        user_id: user_id.to_string(),
        weeks,
        rotation_state: RotationState {
            cycle_number: 1,
            cycle_started_at: Utc::now().to_rfc3339(),
            used_recipe_ids: HashSet::new(),
            total_favorite_count: recipe_ids.len(),
            used_main_course_ids: Vec::new(),
            used_appetizer_ids: Vec::new(),
            used_dessert_ids: Vec::new(),
            cuisine_usage_count: HashMap::new(),
            last_complex_meal_date: None,
        },
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::save::<MealPlanAggregate>(&generation_batch_id)
        .data(&event_data)?
        .metadata(&true)?
        .commit(&executor)
        .await?;

    // Process projection synchronously
    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await?;

    Ok(generation_batch_id)
}

/// Helper: Create test app with auth bypass
fn create_test_app(pool: SqlitePool, executor: evento::Sqlite, test_user_id: String) -> Router {
    use axum::{middleware, routing::post};

    let state = AppState {
        db_pool: pool.clone(),
        write_pool: pool.clone(),
        evento_executor: executor,
        jwt_secret: "test_secret".to_string(),
        email_config: imkitchen::email::EmailConfig {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: "test".to_string(),
            smtp_password: "test".to_string(),
            from_email: "test@example.com".to_string(),
            from_name: "Test".to_string(),
            smtp_tls: false,
        },
        base_url: "http://localhost:3000".to_string(),
        stripe_secret_key: "".to_string(),
        stripe_webhook_secret: "".to_string(),
        stripe_price_id: "".to_string(),
        vapid_public_key: "".to_string(),
        generation_locks: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
        bypass_premium: true,
    };

    let test_auth_layer =
        middleware::from_fn(move |mut req: Request, next: axum::middleware::Next| {
            let user_id = test_user_id.clone();
            async move {
                req.extensions_mut().insert(Auth { user_id });
                next.run(req).await
            }
        });

    Router::new()
        .route(
            "/plan/regenerate-all-future",
            post(regenerate_all_future_weeks),
        )
        .layer(test_auth_layer)
        .with_state(state)
}

/// AC-9: Integration test - POST with confirmation regenerates all future weeks
#[tokio::test]
async fn test_regenerate_all_future_weeks_with_confirmation() {
    // Setup
    let pool = create_test_db().await;
    let user_id = "test_user_1";
    create_test_user(&pool, user_id, "test@example.com")
        .await
        .unwrap();

    let recipe_ids = create_test_recipes(&pool, user_id, 10).await.unwrap();
    create_existing_meal_plan(&pool, user_id, &recipe_ids)
        .await
        .unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app(pool.clone(), executor.clone(), user_id.to_string());

    // Count future weeks before regeneration
    let future_weeks_before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM meal_plans WHERE user_id = ?1 AND status = 'future'",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        future_weeks_before, 4,
        "Should have 4 future weeks initially"
    );

    // Make request WITH confirmation
    let request_body = serde_json::json!({ "confirmation": true });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/regenerate-all-future")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert response status
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Expected 200 OK when regenerating with confirmation"
    );

    // Parse response body
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body: serde_json::Value =
        serde_json::from_str(body_str).expect("Response should be valid JSON");

    // AC-8: Verify response structure
    assert!(
        body.get("regenerated_weeks").is_some(),
        "Response should contain regenerated_weeks"
    );
    assert!(
        body.get("preserved_current_week_id").is_some(),
        "Response should contain preserved_current_week_id"
    );
    assert!(
        body.get("first_future_week").is_some(),
        "Response should contain first_future_week"
    );
    assert!(
        body.get("message").is_some(),
        "Response should contain message"
    );

    // Verify regenerated_weeks count
    let regenerated_count = body.get("regenerated_weeks").unwrap().as_i64().unwrap();
    assert_eq!(regenerated_count, 4, "Should regenerate 4 future weeks");

    // AC-3: Verify current week preserved
    let current_week_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM meal_plans WHERE user_id = ?1 AND status = 'current' AND is_locked = 1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(current_week_count, 1, "Current week should be preserved");
}

/// AC-10: Integration test - POST without confirmation returns 400
#[tokio::test]
async fn test_regenerate_all_future_weeks_without_confirmation() {
    // Setup
    let pool = create_test_db().await;
    let user_id = "test_user_2";
    create_test_user(&pool, user_id, "test2@example.com")
        .await
        .unwrap();

    let recipe_ids = create_test_recipes(&pool, user_id, 10).await.unwrap();
    create_existing_meal_plan(&pool, user_id, &recipe_ids)
        .await
        .unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app(pool.clone(), executor.clone(), user_id.to_string());

    // Make request WITHOUT confirmation
    let request_body = serde_json::json!({ "confirmation": false });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/regenerate-all-future")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert error response
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Expected 400 Bad Request without confirmation"
    );

    // Parse response body
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body: serde_json::Value =
        serde_json::from_str(body_str).expect("Error response should be valid JSON");

    // Verify error structure
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("ConfirmationRequired")
    );
    assert!(
        body.get("message").is_some(),
        "Error should contain message"
    );
}

/// AC-2: Test missing confirmation field
#[tokio::test]
async fn test_regenerate_all_future_weeks_missing_confirmation_field() {
    let pool = create_test_db().await;
    let user_id = "test_user_3";
    create_test_user(&pool, user_id, "test3@example.com")
        .await
        .unwrap();

    let recipe_ids = create_test_recipes(&pool, user_id, 10).await.unwrap();
    create_existing_meal_plan(&pool, user_id, &recipe_ids)
        .await
        .unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app(pool.clone(), executor.clone(), user_id.to_string());

    // Make request with EMPTY body (no confirmation field)
    let request_body = serde_json::json!({});

    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/regenerate-all-future")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 400 for missing confirmation
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
