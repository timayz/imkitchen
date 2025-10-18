/// Integration tests for dashboard route (Story 3.9 Review Action Item #3)
///
/// Tests authentication and authorization requirements for the dashboard endpoint.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use evento::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use tower::ServiceExt;

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

/// Test: Dashboard route requires authentication (401 without JWT)
///
/// Verifies that GET /dashboard returns 401 Unauthorized when no JWT token is provided.
/// This ensures the auth middleware is properly protecting the dashboard endpoint.
#[tokio::test]
async fn test_dashboard_requires_authentication() {
    let pool = create_test_db().await;
    let evento_executor: evento::Sqlite = pool.clone().into();

    // Create app state
    let state = imkitchen::routes::AppState {
        db_pool: pool.clone(),
        evento_executor,
        jwt_secret: "test-secret-key-for-testing-only".to_string(),
        email_config: imkitchen::email::EmailConfig {
            smtp_host: "localhost".to_string(),
            smtp_port: 25,
            smtp_username: "test".to_string(),
            smtp_password: "test".to_string(),
            from_email: "test@example.com".to_string(),
            from_name: "Test".to_string(),
        },
        base_url: "http://localhost:3000".to_string(),
        stripe_secret_key: "sk_test_fake".to_string(),
        stripe_webhook_secret: "whsec_test_fake".to_string(),
        stripe_price_id: "price_test_fake".to_string(),
        vapid_public_key: "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nqm50g".to_string(),
        generation_locks: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
    };

    // Build router with auth middleware (same as production)
    let app = axum::Router::new()
        .route(
            "/dashboard",
            axum::routing::get(imkitchen::routes::dashboard_handler),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            imkitchen::middleware::auth_middleware,
        ))
        .with_state(state);

    // Request without authentication cookie
    let request = Request::builder()
        .uri("/dashboard")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Verify: Should return 303 redirect to /login (auth middleware behavior)
    // The auth middleware redirects unauthenticated requests to login page
    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "Dashboard should redirect to login when not authenticated"
    );

    // Verify redirect location
    let location = response
        .headers()
        .get("location")
        .and_then(|h| h.to_str().ok());
    assert_eq!(location, Some("/login"), "Should redirect to /login page");
}

/// Test: Dashboard route rejects invalid JWT tokens
///
/// Verifies that GET /dashboard returns error when JWT token is malformed or invalid.
#[tokio::test]
async fn test_dashboard_rejects_invalid_jwt() {
    let pool = create_test_db().await;
    let evento_executor: evento::Sqlite = pool.clone().into();

    let state = imkitchen::routes::AppState {
        db_pool: pool.clone(),
        evento_executor,
        jwt_secret: "test-secret-key-for-testing-only".to_string(),
        email_config: imkitchen::email::EmailConfig {
            smtp_host: "localhost".to_string(),
            smtp_port: 25,
            smtp_username: "test".to_string(),
            smtp_password: "test".to_string(),
            from_email: "test@example.com".to_string(),
            from_name: "Test".to_string(),
        },
        base_url: "http://localhost:3000".to_string(),
        stripe_secret_key: "sk_test_fake".to_string(),
        stripe_webhook_secret: "whsec_test_fake".to_string(),
        stripe_price_id: "price_test_fake".to_string(),
        vapid_public_key: "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nqm50g".to_string(),
        generation_locks: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
    };

    let app = axum::Router::new()
        .route(
            "/dashboard",
            axum::routing::get(imkitchen::routes::dashboard_handler),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            imkitchen::middleware::auth_middleware,
        ))
        .with_state(state);

    // Request with invalid JWT token
    let request = Request::builder()
        .uri("/dashboard")
        .header("Cookie", "token=invalid.jwt.token.here")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Verify: Should redirect to login (invalid token treated as unauthenticated)
    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "Dashboard should redirect to login with invalid JWT"
    );

    let location = response
        .headers()
        .get("location")
        .and_then(|h| h.to_str().ok());
    assert_eq!(location, Some("/login"), "Should redirect to /login page");
}
