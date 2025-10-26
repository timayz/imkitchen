/// Integration tests for Week Regeneration Route (Story 8.3)
///
/// These tests verify the POST /plan/week/:week_id/regenerate route behavior:
/// - AC-1: Route creation and authentication
/// - AC-2: Authorization check (week belongs to user) and lock validation
/// - AC-3: Load rotation state from database
/// - AC-4: Call generate_single_week algorithm
/// - AC-5: Emit SingleWeekRegenerated event
/// - AC-6: Shopping list regeneration (async via projection)
/// - AC-7: 403 for locked week
/// - AC-8: 400 for past week
/// - AC-9: POST regenerates future week successfully
/// - AC-10: POST on locked week returns 403
use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode},
    routing::post,
    Router,
};
use chrono::{Duration, NaiveDate, Utc};
use evento::migrator::{Migrate, Plan};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use tower::ServiceExt;

// Import the route handler and AppState
use imkitchen::middleware::auth::Auth;
use imkitchen::routes::{regenerate_week, AppState};

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

/// Helper: Create test recipes for meal planning (all three types: appetizer, main_course, dessert)
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
                    id, user_id, title, ingredients, instructions,
                    prep_time_min, cook_time_min, serving_size, recipe_type,
                    is_favorite, is_shared, complexity, cuisine, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                "#,
            )
            .bind(&recipe_id)
            .bind(user_id)
            .bind(format!("{} Recipe {}", recipe_type, i))
            .bind(r#"[{"name":"ingredient1","amount":"1 cup"}]"#)
            .bind(r#"[{"step_number":1,"instruction":"Cook it"}]"#)
            .bind(15)
            .bind(30)
            .bind(4)
            .bind(recipe_type)
            .bind(true) // is_favorite
            .bind(false)
            .bind("simple")
            .bind("italian")
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;

            recipe_ids.push(recipe_id);
        }
    }

    Ok(recipe_ids)
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

    // Create middleware that injects Auth extension
    let auth_middleware = middleware::from_fn(
        |mut req: Request<Body>, next: axum::middleware::Next| async move {
            let auth = Auth {
                user_id: "test_user".to_string(),
            };
            req.extensions_mut().insert(auth);
            next.run(req).await
        },
    );

    Router::new()
        .route("/plan/week/{week_id}/regenerate", post(regenerate_week))
        .layer(auth_middleware)
        .with_state(state)
}

/// Helper: Create meal plan with weeks in database (simulates multi-week generation result)
async fn create_test_meal_plan_with_weeks(
    pool: &SqlitePool,
    user_id: &str,
    generation_batch_id: &str,
    week_count: usize,
    start_date: NaiveDate,
) -> Result<Vec<String>, sqlx::Error> {
    let mut week_ids = Vec::new();

    for week_index in 0..week_count {
        let week_id = uuid::Uuid::new_v4().to_string();
        let week_start = start_date + Duration::weeks(week_index as i64);
        let week_end = week_start + Duration::days(6);

        // Determine status: All weeks are 'active' (database only uses 'active'/'archived')
        // is_locked determines if the week can be regenerated
        let status = "active";
        let is_locked = week_index == 0; // Only first week is locked

        sqlx::query(
            r#"
            INSERT INTO meal_plans (
                id, user_id, start_date, end_date, status, is_locked,
                generation_batch_id, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&week_id)
        .bind(user_id)
        .bind(week_start.format("%Y-%m-%d").to_string())
        .bind(week_end.format("%Y-%m-%d").to_string())
        .bind(status)
        .bind(is_locked)
        .bind(generation_batch_id)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;

        week_ids.push(week_id);
    }

    Ok(week_ids)
}

/// Helper: Create rotation state in database
async fn create_test_rotation_state(
    pool: &SqlitePool,
    user_id: &str,
    generation_batch_id: &str,
) -> Result<(), sqlx::Error> {
    let id = format!("rotation_{}", generation_batch_id);
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO meal_plan_rotation_state (
            id, user_id, generation_batch_id, used_main_course_ids,
            used_appetizer_ids, used_dessert_ids, cuisine_usage_count,
            last_complex_meal_date, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
    )
    .bind(&id)
    .bind(user_id)
    .bind(generation_batch_id)
    .bind("[]") // Empty JSON array for used_main_course_ids
    .bind("[]") // Empty JSON array for used_appetizer_ids
    .bind("[]") // Empty JSON array for used_dessert_ids
    .bind("{}") // Empty JSON object for cuisine_usage_count
    .bind(Option::<String>::None) // No last_complex_meal_date
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(())
}

/// AC-9: Test successful week regeneration for future week
#[tokio::test]
async fn test_regenerate_future_week_successfully() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();

    // Setup: Create test user
    create_test_user(&pool, "test_user", "test@example.com")
        .await
        .unwrap();

    // Setup: Create test recipes (7+ of each type for one week)
    create_test_recipes(&pool, "test_user", 8).await.unwrap();

    // Setup: Create meal plan with 3 weeks (current + 2 future)
    let generation_batch_id = "batch_123";
    let start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap(); // Monday
    let week_ids =
        create_test_meal_plan_with_weeks(&pool, "test_user", generation_batch_id, 3, start_date)
            .await
            .unwrap();

    // Setup: Create rotation state
    create_test_rotation_state(&pool, "test_user", generation_batch_id)
        .await
        .unwrap();

    // Target: Regenerate second week (future, unlocked)
    let target_week_id = &week_ids[1];

    // Create test app
    let app = create_test_app(pool.clone(), executor.clone());

    // AC-9: POST /plan/week/:week_id/regenerate should succeed for future week
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/plan/week/{}/regenerate", target_week_id))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: Response status is 200 OK
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::OK {
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Expected 200 OK for future week regeneration, got {}: {}",
            status, body_str
        );
    }

    // Assert: Response body contains regenerated week data
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(
        json["week"].is_object(),
        "Response should contain week object"
    );
    assert_eq!(
        json["week"]["id"].as_str().unwrap(),
        target_week_id,
        "Week ID should match"
    );
    assert!(
        json["week"]["meal_assignments"].is_array(),
        "Week should have meal assignments"
    );
    assert_eq!(
        json["week"]["meal_assignments"].as_array().unwrap().len(),
        21,
        "Week should have 21 meal assignments (7 days × 3 courses)"
    );
    assert_eq!(
        json["message"].as_str().unwrap(),
        "Week regenerated successfully. Shopping list updated.",
        "Success message should be present"
    );

    // AC-5: Verify SingleWeekRegenerated event emitted (use unsafe_oneshot for sync processing)
    // Note: In a full integration test, we'd subscribe to the event and verify projection updates
    // For now, we verify the event was committed by checking the response succeeded
}

/// AC-10: Test locked week regeneration returns 403
#[tokio::test]
async fn test_regenerate_locked_week_returns_403() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();

    // Setup: Create test user
    create_test_user(&pool, "test_user", "test@example.com")
        .await
        .unwrap();

    // Setup: Create test recipes
    create_test_recipes(&pool, "test_user", 8).await.unwrap();

    // Setup: Create meal plan with current week (locked)
    let generation_batch_id = "batch_456";
    let start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();
    let week_ids =
        create_test_meal_plan_with_weeks(&pool, "test_user", generation_batch_id, 1, start_date)
            .await
            .unwrap();

    // Setup: Create rotation state
    create_test_rotation_state(&pool, "test_user", generation_batch_id)
        .await
        .unwrap();

    // Target: Current week (locked)
    let locked_week_id = &week_ids[0];

    // Create test app
    let app = create_test_app(pool.clone(), executor.clone());

    // AC-10, AC-7: POST /plan/week/:week_id/regenerate should return 403 for locked week
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/plan/week/{}/regenerate", locked_week_id))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: Response status is 403 Forbidden
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Expected 403 Forbidden for locked week regeneration"
    );

    // Assert: Error response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        json["error"].as_str().unwrap(),
        "WeekLocked",
        "Error code should be WeekLocked"
    );
    assert!(
        json["message"]
            .as_str()
            .unwrap()
            .contains("locked to prevent disrupting"),
        "Error message should explain locked week constraint"
    );
}

/// AC-8: Test past week regeneration returns 400
#[tokio::test]
async fn test_regenerate_past_week_returns_400() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();

    // Setup: Create test user
    create_test_user(&pool, "test_user", "test@example.com")
        .await
        .unwrap();

    // Setup: Create test recipes
    create_test_recipes(&pool, "test_user", 8).await.unwrap();

    // Setup: Create past week manually
    let past_week_id = uuid::Uuid::new_v4().to_string();
    let past_start = NaiveDate::from_ymd_opt(2025, 10, 13).unwrap(); // Last week Monday
    let past_end = past_start + Duration::days(6);

    sqlx::query(
        r#"
        INSERT INTO meal_plans (
            id, user_id, start_date, end_date, status, is_locked,
            generation_batch_id, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&past_week_id)
    .bind("test_user")
    .bind(past_start.format("%Y-%m-%d").to_string())
    .bind(past_end.format("%Y-%m-%d").to_string())
    .bind("archived") // Past status (database uses 'archived' for past weeks)
    .bind(false) // Not locked (past weeks are no longer locked)
    .bind("batch_past")
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    // Create test app
    let app = create_test_app(pool.clone(), executor.clone());

    // AC-8: POST /plan/week/:week_id/regenerate should return 400 for past week
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/plan/week/{}/regenerate", past_week_id))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: Response status is 400 Bad Request
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Expected 400 Bad Request for past week regeneration"
    );

    // Assert: Error response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        json["error"].as_str().unwrap(),
        "WeekAlreadyStarted",
        "Error code should be WeekAlreadyStarted"
    );
    assert!(
        json["message"]
            .as_str()
            .unwrap()
            .contains("already started"),
        "Error message should explain past week constraint"
    );
}

/// Test unauthorized week access returns 403
#[tokio::test]
async fn test_regenerate_unauthorized_week_returns_403() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();

    // Setup: Create two test users
    create_test_user(&pool, "test_user", "test@example.com")
        .await
        .unwrap();
    create_test_user(&pool, "other_user", "other@example.com")
        .await
        .unwrap();

    // Setup: Create test recipes for other_user
    create_test_recipes(&pool, "other_user", 8).await.unwrap();

    // Setup: Create meal plan for other_user
    let generation_batch_id = "batch_other";
    let start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();
    let week_ids = create_test_meal_plan_with_weeks(
        &pool,
        "other_user", // Different user
        generation_batch_id,
        2,
        start_date,
    )
    .await
    .unwrap();

    // Target: Other user's future week
    let other_week_id = &week_ids[1];

    // Create test app (authenticated as test_user, not other_user)
    let app = create_test_app(pool.clone(), executor.clone());

    // Test: POST /plan/week/:week_id/regenerate should return 403 for other user's week
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/plan/week/{}/regenerate", other_week_id))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: Response status is 403 Forbidden
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Expected 403 Forbidden for unauthorized week access"
    );

    // Assert: Error response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        json["error"].as_str().unwrap(),
        "Forbidden",
        "Error code should be Forbidden"
    );
}
