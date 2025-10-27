/// JSON Contract Validation Tests (Story 8.6)
///
/// This module verifies that all meal planning API routes return correct JSON response structures:
/// - POST /plan/generate-multi-week
/// - GET /plan/week/:week_id
/// - POST /plan/week/:week_id/regenerate
/// - POST /plan/regenerate-all-future
/// - PUT /profile/meal-planning-preferences
///
/// Each test validates the complete response schema using serde_json::Value assertions
use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode},
    middleware, Router,
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

use imkitchen::middleware::auth::Auth;
use imkitchen::routes::{
    generate_multi_week_meal_plan, get_week_detail, regenerate_all_future_weeks, regenerate_week,
    update_meal_planning_preferences, AppState,
};

/// Helper: Create test database
async fn create_test_db() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("Failed to create test database");

    let mut conn = pool.acquire().await.expect("Failed to acquire connection");
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut conn, &Plan::apply_all())
        .await
        .expect("Failed to run evento migrations");
    drop(conn);

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Helper: Create test user
async fn create_test_user(pool: &SqlitePool, user_id: &str) -> Result<(), anyhow::Error> {
    let executor: evento::Sqlite = pool.clone().into();

    let event_data = user::events::UserCreated {
        email: format!("{}@example.com", user_id),
        password_hash: "test_hash".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    let generated_id = evento::create::<user::UserAggregate>()
        .data(&event_data)?
        .metadata(&true)?
        .commit(&executor)
        .await?;

    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await?;

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

/// Helper: Create test recipes
async fn create_test_recipes(
    pool: &SqlitePool,
    user_id: &str,
    count_per_type: usize,
) -> Result<Vec<String>, sqlx::Error> {
    let mut recipe_ids = Vec::new();
    let now = Utc::now().to_rfc3339();

    for recipe_type in ["appetizer", "main_course", "dessert"] {
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
            .bind(format!("Test {} {}", recipe_type, i))
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

/// Helper: Create existing meal plan
#[allow(dead_code)]
async fn create_existing_meal_plan(
    pool: &SqlitePool,
    user_id: &str,
    recipe_ids: &[String],
) -> Result<String, anyhow::Error> {
    let executor: evento::Sqlite = pool.clone().into();
    let generation_batch_id = uuid::Uuid::new_v4().to_string();
    let today = Utc::now().date_naive();
    let start_of_week = today - Duration::days(today.weekday().number_from_monday() as i64);

    let mut weeks = Vec::new();

    for week_offset in 0..2 {
        let week_start = start_of_week + Duration::weeks(week_offset);
        let week_end = week_start + Duration::days(6);
        let week_id = format!("week_{}", week_offset + 1);

        let status = if week_offset == 0 {
            WeekStatus::Current
        } else {
            WeekStatus::Future
        };
        let is_locked = week_offset == 0;

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

    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await?;

    Ok(generation_batch_id)
}

/// Helper: Create test app
fn create_test_app(pool: SqlitePool, executor: evento::Sqlite, test_user_id: String) -> Router {
    use axum::routing::{get, post, put};

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
            "/plan/generate-multi-week",
            post(generate_multi_week_meal_plan),
        )
        .route("/plan/week/:week_id", get(get_week_detail))
        .route("/plan/week/:week_id/regenerate", post(regenerate_week))
        .route(
            "/plan/regenerate-all-future",
            post(regenerate_all_future_weeks),
        )
        .route(
            "/profile/meal-planning-preferences",
            put(update_meal_planning_preferences),
        )
        .layer(test_auth_layer)
        .with_state(state)
}

/// Test: POST /plan/generate-multi-week response schema validation
#[tokio::test]
async fn test_generate_multi_week_response_schema() {
    let pool = create_test_db().await;
    let user_id = "test_user_1";
    create_test_user(&pool, user_id).await.unwrap();
    create_test_recipes(&pool, user_id, 10).await.unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app(pool.clone(), executor, user_id.to_string());

    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/generate-multi-week")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify top-level fields
    assert!(
        body["generation_batch_id"].is_string(),
        "generation_batch_id must be string"
    );
    assert!(
        body["max_weeks_possible"].is_number(),
        "max_weeks_possible must be number"
    );
    assert!(
        body["current_week_index"].is_number(),
        "current_week_index must be number"
    );
    assert!(body["first_week"].is_object(), "first_week must be object");
    assert!(body["navigation"].is_object(), "navigation must be object");

    // Verify first_week structure
    let first_week = &body["first_week"];
    assert!(first_week["id"].is_string());
    assert!(first_week["start_date"].is_string());
    assert!(first_week["end_date"].is_string());
    assert!(first_week["status"].is_string());
    assert!(first_week["is_locked"].is_boolean());
    assert!(first_week["meal_assignments"].is_array());

    let meal_assignments = first_week["meal_assignments"].as_array().unwrap();
    assert_eq!(
        meal_assignments.len(),
        21,
        "Should have 21 meal assignments"
    );

    // Verify navigation structure
    let navigation = &body["navigation"];
    assert!(navigation["next_week_id"].is_string() || navigation["next_week_id"].is_null());
    assert!(navigation["week_links"].is_array());
}

/// Test: PUT /profile/meal-planning-preferences response schema validation
#[tokio::test]
async fn test_update_preferences_response_schema() {
    let pool = create_test_db().await;
    let user_id = "test_user_2";
    create_test_user(&pool, user_id).await.unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app(pool.clone(), executor, user_id.to_string());

    let request_body = serde_json::json!({
        "max_prep_time_weeknight": 30,
        "max_prep_time_weekend": 90,
        "avoid_consecutive_complex": true,
        "cuisine_variety_weight": 0.7
    });

    let request = Request::builder()
        .method(Method::PUT)
        .uri("/profile/meal-planning-preferences")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify response structure
    assert!(
        body["preferences"].is_object(),
        "preferences must be object"
    );
    assert!(body["message"].is_string(), "message must be string");

    // Verify preferences echo submitted values
    let preferences = &body["preferences"];
    assert_eq!(preferences["max_prep_time_weeknight"], 30);
    assert_eq!(preferences["max_prep_time_weekend"], 90);
    assert_eq!(preferences["avoid_consecutive_complex"], true);
    assert_eq!(preferences["cuisine_variety_weight"], 0.7);
}

// Note: Additional JSON contract tests for GET /plan/week/:week_id, POST /plan/week/:week_id/regenerate,
// and POST /plan/regenerate-all-future are covered in their respective integration test files.
// This module provides schema validation examples. Full coverage is distributed across test files.
