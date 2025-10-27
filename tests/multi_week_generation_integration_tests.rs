/// Integration tests for Multi-Week Generation Route (Story 8.1)
///
/// These tests verify the POST /plan/generate-multi-week route behavior:
/// - AC-1: Route POST /plan/generate-multi-week created
/// - AC-2: Route protected by authentication middleware (JWT cookie)
/// - AC-3: Handler extracts user_id from JWT claims
/// - AC-4: Handler loads user's favorite recipes from database
/// - AC-5: Handler loads user's meal planning preferences from users table
/// - AC-6: Handler calls generate_multi_week_meal_plans algorithm
/// - AC-7: Handler commits MultiWeekMealPlanGenerated event to evento
/// - AC-8: Handler returns JSON with first week data + navigation links
/// - AC-9: Error: InsufficientRecipes returns 400 with helpful message + action
/// - AC-10: Error: AlgorithmTimeout returns 500 with retry message
/// - AC-11: Integration test: POST generates meal plan, verifies JSON response structure
use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode},
    Router,
};
use chrono::Utc;
use evento::migrator::{Migrate, Plan};
use http_body_util::BodyExt;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use tower::ServiceExt;

// Import the route handler and AppState
use imkitchen::middleware::auth::Auth;
use imkitchen::routes::{generate_multi_week_meal_plan, AppState};

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

/// Helper: Create test user using evento properly
async fn create_test_user(
    pool: &SqlitePool,
    user_id: &str,
    email: &str,
) -> Result<(), anyhow::Error> {
    let now = Utc::now().to_rfc3339();

    // Create evento executor from pool
    let executor: evento::Sqlite = pool.clone().into();

    // Create UserCreated event
    let event_data = user::events::UserCreated {
        email: email.to_string(),
        password_hash: "test_hash".to_string(),
        created_at: now.clone(),
    };

    // Use evento::create to properly create the UserAggregate
    let generated_id = evento::create::<user::UserAggregate>()
        .data(&event_data)
        .expect("Failed to encode UserCreated event data")
        .metadata(&true)
        .expect("Failed to encode event metadata")
        .commit(&executor)
        .await
        .expect("Failed to commit UserCreated event");

    // Process user projection synchronously using unsafe_oneshot
    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .expect("Failed to process user projection");

    // Update IDs to use predictable test user_id
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

/// Helper: Create test recipes with all three types (appetizer, main_course, dessert)
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
            .bind(0) // is_shared (INTEGER not boolean)
            .bind(1) // is_favorite (INTEGER not boolean, IMPORTANT: all test recipes are favorites)
            .bind(&now)
            .bind(&now)
            .bind(recipe_type) // recipe_type
            .bind("italian") // cuisine
            .bind("[]") // dietary_tags
            .bind("[]") // ingredients (JSON)
            .bind("[]") // instructions (JSON)
            .execute(pool)
            .await?;

            recipe_ids.push(recipe_id);
        }
    }

    Ok(recipe_ids)
}

/// Helper: Create test app with auth middleware bypass
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

    // Middleware to bypass auth and inject test user
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
        .layer(test_auth_layer)
        .with_state(state)
}

/// AC-11: Integration test - POST generates meal plan and verifies JSON response structure
#[tokio::test]
async fn test_generate_multi_week_with_sufficient_recipes() {
    // Setup
    let pool = create_test_db().await;
    let user_id = "test_user_1";
    create_test_user(&pool, user_id, "test@example.com")
        .await
        .unwrap();

    // Create 15 recipes per type (45 total, all favorites)
    // Algorithm requires at least 7 unused main courses per week for multi-week generation
    // For 5 weeks, need substantial recipe pool (15 per category ensures enough variety)
    create_test_recipes(&pool, user_id, 15).await.unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app(pool.clone(), executor.clone(), user_id.to_string());

    // Make request
    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/generate-multi-week")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Parse response body and assert status
    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();

    if status != StatusCode::OK {
        panic!(
            "Expected 200 OK with sufficient recipes, got {}: {}",
            status, body_str
        );
    }

    let body: serde_json::Value =
        serde_json::from_str(body_str).expect("Response should be valid JSON");

    // AC-8: Verify JSON response structure
    assert!(
        body.get("generation_batch_id").is_some(),
        "Response should contain generation_batch_id"
    );
    assert!(
        body.get("max_weeks_possible").is_some(),
        "Response should contain max_weeks_possible"
    );
    assert!(
        body.get("current_week_index").is_some(),
        "Response should contain current_week_index"
    );
    assert!(
        body.get("first_week").is_some(),
        "Response should contain first_week"
    );
    assert!(
        body.get("navigation").is_some(),
        "Response should contain navigation"
    );

    // Verify first_week structure
    let first_week = body.get("first_week").unwrap();
    assert!(first_week.get("id").is_some());
    assert!(first_week.get("start_date").is_some());
    assert!(first_week.get("end_date").is_some());
    assert!(first_week.get("status").is_some());
    assert!(first_week.get("is_locked").is_some());
    assert!(first_week.get("meal_assignments").is_some());

    // Verify meal_assignments is an array of 21 items (7 days × 3 meals)
    let meal_assignments = first_week
        .get("meal_assignments")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(
        meal_assignments.len(),
        21,
        "Should have 21 meal assignments (7 days × 3 meals)"
    );

    // Verify navigation structure
    let navigation = body.get("navigation").unwrap();
    assert!(navigation.get("next_week_id").is_some());
    assert!(navigation.get("week_links").is_some());

    // AC-7: Process MultiWeekMealPlanGenerated event through projection synchronously
    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify read model updated (meal plans table)
    let meal_plan_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM meal_plans WHERE user_id = ?1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert!(
        meal_plan_count > 0,
        "Meal plans should be created in read model"
    );
}

/// AC-9: Test InsufficientRecipes error response
#[tokio::test]
async fn test_generate_multi_week_with_insufficient_recipes() {
    // Setup
    let pool = create_test_db().await;
    let user_id = "test_user_2";
    create_test_user(&pool, user_id, "test2@example.com")
        .await
        .unwrap();

    // Create only 3 recipes per type (9 total, insufficient for multi-week generation)
    create_test_recipes(&pool, user_id, 3).await.unwrap();

    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app(pool.clone(), executor.clone(), user_id.to_string());

    // Make request
    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/generate-multi-week")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert error response
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Expected 400 Bad Request with insufficient recipes"
    );

    // Parse response body
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body: serde_json::Value =
        serde_json::from_str(body_str).expect("Error response should be valid JSON");

    // Verify error structure
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("InsufficientRecipes")
    );
    assert!(
        body.get("message").is_some(),
        "Error should contain message"
    );
    assert!(
        body.get("details").is_some(),
        "Error should contain details with category counts"
    );
    assert!(
        body.get("action").is_some(),
        "Error should contain action with 'Add More Recipes' CTA"
    );

    // Verify action contains URL
    let action = body.get("action").unwrap();
    assert!(action.get("label").is_some());
    assert!(action.get("url").is_some());
}

/// AC-2, AC-3: Test authentication requirement (implicit via test helper auth bypass)
/// Note: Full auth tests are in test_authentication_authorization.rs
#[tokio::test]
async fn test_generate_multi_week_requires_authentication() {
    // This test verifies that the route handler expects Auth extension to be present
    // In production, auth_middleware injects Auth extension from JWT cookie
    // Test bypasses auth_middleware but still requires Auth extension injection

    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();

    // Create app WITHOUT auth bypass middleware
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

    use axum::routing::post;
    let app = Router::new()
        .route(
            "/plan/generate-multi-week",
            post(generate_multi_week_meal_plan),
        )
        .with_state(state);

    // Make request WITHOUT auth extension
    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/generate-multi-week")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Handler panics if Auth extension is missing (which gets caught by Axum as 500)
    // In production, auth_middleware returns 401 before reaching handler
    assert_ne!(
        response.status(),
        StatusCode::OK,
        "Request without auth should not succeed"
    );
}
