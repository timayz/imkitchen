/// Integration tests for Week Navigation Route (Story 8.2)
///
/// These tests verify the GET /plan/week/:week_id route behavior:
/// - AC-1, AC-2: Route creation and authentication
/// - AC-3: Authorization check (week belongs to user)
/// - AC-4: Load week data from read models
/// - AC-5: Load shopping list
/// - AC-6: JSON response structure
/// - AC-7: 404 for invalid week_id
/// - AC-8: 403 for unauthorized access
/// - AC-9, AC-10: Integration test coverage
use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode},
    routing::get,
    Router,
};
use chrono::Utc;
use evento::migrator::{Migrate, Plan};
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
use imkitchen::routes::{get_week_detail, AppState};

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
                prep_time_min, cook_time_min, serving_size, recipe_type,
                is_favorite, is_shared, complexity, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
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
        .bind("main_course") // recipe_type
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

/// Helper: Generate test JWT for authenticated requests
fn create_test_jwt(user_id: &str) -> String {
    // Simple mock JWT - in real implementation would use proper JWT signing
    // For testing, we'll use Auth middleware bypass pattern
    format!("test_jwt_{}", user_id)
}

/// Helper: Create test app with auth middleware bypass for integration tests
fn create_test_app(pool: SqlitePool, executor: evento::Sqlite) -> Router {
    use axum::middleware;

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

    // Create router with auth middleware bypass for testing
    Router::new()
        .route("/plan/week/{week_id}", get(get_week_detail))
        .layer(middleware::from_fn(
            |mut req: Request, next: middleware::Next| async move {
                // Extract user_id from test JWT in cookie header
                let auth_header = req.headers().get(header::COOKIE);
                if let Some(cookie) = auth_header {
                    if let Ok(cookie_str) = cookie.to_str() {
                        if cookie_str.starts_with("test_jwt_") {
                            let user_id = cookie_str.strip_prefix("test_jwt_").unwrap().to_string();
                            req.extensions_mut().insert(Auth { user_id });
                        }
                    }
                }
                next.run(req).await
            },
        ))
        .with_state(state)
}

/// Test AC-9: GET with valid week_id returns correct data
#[tokio::test]
async fn test_get_week_detail_with_valid_week_id() {
    // Setup: Create test database and evento executor
    let pool = create_test_db().await;
    let user_id = "user_test_1";

    create_test_user(&pool, user_id, "test@example.com")
        .await
        .expect("Failed to create test user");

    let evento_executor: evento::Sqlite = pool.clone().into();

    // Create test recipes
    create_test_recipes(&pool, user_id, 21)
        .await
        .expect("Failed to create test recipes");

    // Generate meal plan with MultiWeekMealPlanGenerated event
    let generation_batch_id = uuid::Uuid::new_v4().to_string();
    let week_id = uuid::Uuid::new_v4().to_string();
    // Use dates far in the future to ensure "future" status
    let start_date = "2026-10-28"; // Monday
    let end_date = "2026-11-03"; // Sunday

    let meal_assignments: Vec<MealAssignment> = (0..21)
        .map(|i| {
            let day_offset = i / 3;
            let course_type = match i % 3 {
                0 => "appetizer",
                1 => "main_course",
                _ => "dessert",
            };
            let date = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
                .unwrap()
                .checked_add_signed(chrono::Duration::days(day_offset))
                .unwrap()
                .format("%Y-%m-%d")
                .to_string();

            MealAssignment {
                date,
                course_type: course_type.to_string(),
                recipe_id: format!("recipe_{}", (i % 21) + 1),
                prep_required: i % 5 == 0,
                assignment_reasoning: Some(format!("Test reasoning {}", i)),
                accompaniment_recipe_id: None,
            }
        })
        .collect();

    let week_data = WeekMealPlanData {
        id: week_id.clone(),
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
        status: WeekStatus::Current,
        is_locked: false,
        meal_assignments: meal_assignments.clone(),
        shopping_list_id: uuid::Uuid::new_v4().to_string(),
    };

    let rotation_state = RotationState {
        cycle_number: 1,
        cycle_started_at: Utc::now().to_rfc3339(),
        used_recipe_ids: HashSet::new(),
        total_favorite_count: 21,
        used_main_course_ids: vec![],
        used_appetizer_ids: vec![],
        used_dessert_ids: vec![],
        cuisine_usage_count: HashMap::new(),
        last_complex_meal_date: None,
    };

    let event_data = MultiWeekMealPlanGenerated {
        generation_batch_id: generation_batch_id.clone(),
        user_id: user_id.to_string(),
        weeks: vec![week_data],
        rotation_state,
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

    // Use unsafe_oneshot to synchronously process the event projection
    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&evento_executor)
        .await
        .expect("Failed to process meal plan projection");

    // Create test app
    let app = create_test_app(pool.clone(), evento_executor);

    // Act: Make GET request to /plan/week/:week_id
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/plan/week/{}", week_id))
                .header(header::COOKIE, create_test_jwt(user_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: Verify response status is 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Parse JSON response
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify JSON structure (AC-6)
    assert!(response_json.get("week").is_some());
    assert!(response_json.get("navigation").is_some());

    let week = response_json.get("week").unwrap();
    assert_eq!(week.get("id").unwrap().as_str().unwrap(), week_id);
    assert_eq!(
        week.get("start_date").unwrap().as_str().unwrap(),
        start_date
    );
    assert_eq!(week.get("end_date").unwrap().as_str().unwrap(), end_date);
    assert_eq!(week.get("status").unwrap().as_str().unwrap(), "active"); // DB stores "active" for future weeks
    assert!(!week.get("is_locked").unwrap().as_bool().unwrap());

    // Verify meal_assignments array has 21 items
    let meal_assignments = week.get("meal_assignments").unwrap().as_array().unwrap();
    assert_eq!(
        meal_assignments.len(),
        21,
        "Should have 21 meal assignments (7 days × 3 courses)"
    );

    // Verify navigation links
    let navigation = response_json.get("navigation").unwrap();
    assert!(navigation.get("previous_week_id").is_some());
    assert!(navigation.get("next_week_id").is_some());
}

/// Test AC-10: GET with invalid week_id returns 404
#[tokio::test]
async fn test_get_week_detail_with_invalid_week_id() {
    // Setup
    let pool = create_test_db().await;
    let user_id = "user_test_2";

    create_test_user(&pool, user_id, "test2@example.com")
        .await
        .expect("Failed to create test user");

    let evento_executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app(pool.clone(), evento_executor);

    // Act: Make GET request with non-existent week_id
    let invalid_week_id = uuid::Uuid::new_v4().to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/plan/week/{}", invalid_week_id))
                .header(header::COOKIE, create_test_jwt(user_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: Verify response status is 404 Not Found (AC-7)
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Parse error JSON
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify error code
    assert_eq!(
        error_json.get("error").unwrap().as_str().unwrap(),
        "WeekNotFound"
    );
}

/// Test AC-8: GET with week belonging to different user returns 403
#[tokio::test]
async fn test_get_week_detail_authorization_failure() {
    // Setup: Create two users
    let pool = create_test_db().await;
    let user_a = "user_a";
    let user_b = "user_b";

    create_test_user(&pool, user_a, "usera@example.com")
        .await
        .expect("Failed to create user A");
    create_test_user(&pool, user_b, "userb@example.com")
        .await
        .expect("Failed to create user B");

    let evento_executor: evento::Sqlite = pool.clone().into();

    // Create recipes for user A
    create_test_recipes(&pool, user_a, 21)
        .await
        .expect("Failed to create test recipes");

    // Generate meal plan for user A
    let generation_batch_id = uuid::Uuid::new_v4().to_string();
    let week_id = uuid::Uuid::new_v4().to_string();
    let start_date = "2026-10-28"; // Future date
    let end_date = "2026-11-03";

    let meal_assignments: Vec<MealAssignment> = (0..21)
        .map(|i| {
            let day_offset = i / 3;
            let course_type = match i % 3 {
                0 => "appetizer",
                1 => "main_course",
                _ => "dessert",
            };
            let date = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
                .unwrap()
                .checked_add_signed(chrono::Duration::days(day_offset))
                .unwrap()
                .format("%Y-%m-%d")
                .to_string();

            MealAssignment {
                date,
                course_type: course_type.to_string(),
                recipe_id: format!("recipe_{}", (i % 21) + 1),
                prep_required: false,
                assignment_reasoning: None,
                accompaniment_recipe_id: None,
            }
        })
        .collect();

    let week_data = WeekMealPlanData {
        id: week_id.clone(),
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
        status: WeekStatus::Current,
        is_locked: false,
        meal_assignments,
        shopping_list_id: uuid::Uuid::new_v4().to_string(),
    };

    let rotation_state = RotationState {
        cycle_number: 1,
        cycle_started_at: Utc::now().to_rfc3339(),
        used_recipe_ids: HashSet::new(),
        total_favorite_count: 21,
        used_main_course_ids: vec![],
        used_appetizer_ids: vec![],
        used_dessert_ids: vec![],
        cuisine_usage_count: HashMap::new(),
        last_complex_meal_date: None,
    };

    let event_data = MultiWeekMealPlanGenerated {
        generation_batch_id,
        user_id: user_a.to_string(), // Belongs to user A
        weeks: vec![week_data],
        rotation_state,
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

    // Create test app
    let app = create_test_app(pool.clone(), evento_executor);

    // Act: User B tries to access user A's week
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/plan/week/{}", week_id))
                .header(header::COOKIE, create_test_jwt(user_b)) // User B auth
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: Verify response status is 403 Forbidden (AC-8)
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Parse error JSON
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify error code
    assert_eq!(
        error_json.get("error").unwrap().as_str().unwrap(),
        "Forbidden"
    );
}

/// Test: Invalid UUID format returns 400 Bad Request
#[tokio::test]
async fn test_get_week_detail_invalid_uuid_format() {
    // Setup
    let pool = create_test_db().await;
    let user_id = "user_test_3";

    create_test_user(&pool, user_id, "test3@example.com")
        .await
        .expect("Failed to create test user");

    let evento_executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app(pool.clone(), evento_executor);

    // Act: Make GET request with invalid UUID format
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/plan/week/invalid-uuid-format")
                .header(header::COOKIE, create_test_jwt(user_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: Verify response status is 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Parse error JSON
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify error code
    assert_eq!(
        error_json.get("error").unwrap().as_str().unwrap(),
        "BadRequest"
    );
    assert!(error_json
        .get("message")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("UUID"));
}
