/// Consolidated Authentication and Authorization Integration Tests (Story 8.6)
///
/// This module consolidates authentication and authorization tests for all meal planning API routes:
/// - POST /plan/generate-multi-week
/// - GET /plan/week/:week_id
/// - POST /plan/week/:week_id/regenerate
/// - POST /plan/regenerate-all-future
/// - PUT /profile/meal-planning-preferences
///
/// Test Coverage:
/// - 401 Unauthorized when JWT cookie missing
/// - 403 Forbidden when resource belongs to different user
use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode},
    middleware,
    routing::{get, post, put},
    Router,
};
use evento::migrator::{Migrate, Plan};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use tower::ServiceExt;

use imkitchen::middleware::auth_middleware;
use imkitchen::routes::{
    generate_multi_week_meal_plan, get_week_detail, regenerate_all_future_weeks, regenerate_week,
    update_meal_planning_preferences, AppState,
};

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

/// Helper: Create test app with REAL auth middleware (no bypass)
fn create_test_app_with_real_auth(pool: SqlitePool, executor: evento::Sqlite) -> Router {
    let state = AppState {
        db_pool: pool.clone(),
        write_pool: pool.clone(),
        evento_executor: executor,
        jwt_secret: "test_secret_key_minimum_32_characters_long".to_string(),
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

    // Apply REAL auth_middleware to protected routes
    Router::new()
        .route(
            "/plan/generate-multi-week",
            post(generate_multi_week_meal_plan),
        )
        .route("/plan/week/{week_id}", get(get_week_detail))
        .route("/plan/week/{week_id}/regenerate", post(regenerate_week))
        .route(
            "/plan/regenerate-all-future",
            post(regenerate_all_future_weeks),
        )
        .route(
            "/profile/meal-planning-preferences",
            put(update_meal_planning_preferences),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

/// Test: POST /plan/generate-multi-week without JWT returns 303 redirect (to login)
#[tokio::test]
async fn test_generate_multi_week_without_jwt_returns_303() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app_with_real_auth(pool.clone(), executor);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/generate-multi-week")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "POST /plan/generate-multi-week without JWT should return 303 redirect"
    );
}

/// Test: GET /plan/week/:week_id without JWT returns 303 redirect
#[tokio::test]
async fn test_get_week_detail_without_jwt_returns_303() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app_with_real_auth(pool.clone(), executor);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/plan/week/test_week_id")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "GET /plan/week/:week_id without JWT should return 303 redirect"
    );
}

/// Test: POST /plan/week/:week_id/regenerate without JWT returns 303 redirect
#[tokio::test]
async fn test_regenerate_week_without_jwt_returns_303() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app_with_real_auth(pool.clone(), executor);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/week/test_week_id/regenerate")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "POST /plan/week/:week_id/regenerate without JWT should return 303 redirect"
    );
}

/// Test: POST /plan/regenerate-all-future without JWT returns 303 redirect
#[tokio::test]
async fn test_regenerate_all_future_without_jwt_returns_303() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app_with_real_auth(pool.clone(), executor);

    let request_body = serde_json::json!({ "confirmation": true });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/plan/regenerate-all-future")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "POST /plan/regenerate-all-future without JWT should return 303 redirect"
    );
}

/// Test: PUT /profile/meal-planning-preferences without JWT returns 303 redirect
#[tokio::test]
async fn test_update_preferences_without_jwt_returns_303() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();
    let app = create_test_app_with_real_auth(pool.clone(), executor);

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

    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "PUT /profile/meal-planning-preferences without JWT should return 303 redirect"
    );
}

// Note: Cross-user authorization tests (403 Forbidden) are covered in individual route test files:
// - week_navigation_integration_tests.rs: test_get_week_detail_with_different_user_returns_403
// - week_regeneration_integration_tests.rs: test_regenerate_week_with_different_user_returns_403
//
// These tests verify that routes check week.user_id == authenticated user_id and return 403 if mismatch
