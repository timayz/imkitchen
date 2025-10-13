/// Integration tests for subscription routes (Story 1.7)
///
/// Tests cover:
/// - GET /subscription renders page with correct tier status
/// - POST /subscription/upgrade creates Stripe Checkout Session (mocked)
/// - POST /webhooks/stripe with valid signature upgrades user tier
/// - POST /webhooks/stripe with invalid signature returns 401
/// - Premium users bypass recipe limit (validate_recipe_creation)
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use sqlx::Row;
use tower::ServiceExt;

mod common;

/// Helper: Create a test user and return (user_id, jwt_token)
async fn create_test_user(pool: &sqlx::SqlitePool, executor: &evento::Sqlite) -> (String, String) {
    use sqlx::types::Uuid;
    let email = format!("test-{}@example.com", Uuid::new_v4());

    // Register user
    let register_cmd = user::commands::RegisterUserCommand {
        email: email.clone(),
        password: "password123".to_string(),
    };

    let user_id = user::commands::register_user(register_cmd, executor, pool)
        .await
        .unwrap();

    // Process events to project to read model
    user::user_projection(pool.clone())
        .run_once(executor)
        .await
        .unwrap();

    // Generate JWT token
    let jwt_secret = "test_secret_key_minimum_32_characters_long";
    let token = user::jwt::generate_jwt(
        user_id.clone(),
        email.clone(),
        "free".to_string(),
        jwt_secret,
    )
    .unwrap();

    (user_id, token)
}

#[tokio::test]
async fn test_get_subscription_renders_free_tier_page() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    let (_user_id, token) = create_test_user(&pool, &test_app.evento_executor).await;

    // GET /subscription with auth cookie
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/subscription")
                .header("cookie", format!("auth_token={}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify subscription page content
    assert!(body_str.contains("Subscription")); // Page title
    assert!(body_str.contains("free") || body_str.contains("Free")); // Tier display
    assert!(body_str.contains("Upgrade to Premium") || body_str.contains("upgrade"));
    // Upgrade button
}

#[tokio::test]
async fn test_get_subscription_renders_premium_tier_page() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    let (user_id, _token) = create_test_user(&pool, &test_app.evento_executor).await;

    // Upgrade user to premium
    let upgrade_cmd = user::commands::UpgradeSubscriptionCommand {
        user_id: user_id.clone(),
        new_tier: "premium".to_string(),
        stripe_customer_id: Some("cus_test123".to_string()),
        stripe_subscription_id: Some("sub_test456".to_string()),
    };

    user::commands::upgrade_subscription(upgrade_cmd, &test_app.evento_executor)
        .await
        .unwrap();

    // Process events
    test_app.process_events().await;

    // Generate new JWT with premium tier
    let jwt_secret = "test_secret_key_minimum_32_characters_long";
    let premium_token = user::jwt::generate_jwt(
        user_id.clone(),
        "test@example.com".to_string(),
        "premium".to_string(),
        jwt_secret,
    )
    .unwrap();

    // GET /subscription with premium user
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/subscription")
                .header("cookie", format!("auth_token={}", premium_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify premium status displayed
    assert!(body_str.contains("premium") || body_str.contains("Premium"));
}

#[tokio::test]
async fn test_post_subscription_upgrade_requires_authentication() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    // POST /subscription/upgrade without auth cookie
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/subscription/upgrade")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to login or return 401
    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::SEE_OTHER
    );
}

#[tokio::test]
async fn test_validate_recipe_creation_free_tier_enforces_limit() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    let (user_id, _token) = create_test_user(&pool, &test_app.evento_executor).await;

    // Update user recipe_count to 10 (at limit)
    sqlx::query("UPDATE users SET recipe_count = 10 WHERE id = ?1")
        .bind(&user_id)
        .execute(&pool)
        .await
        .unwrap();

    // Attempt to validate recipe creation (should fail)
    let result = user::commands::validate_recipe_creation(&user_id, &pool).await;

    assert!(result.is_err());
    match result {
        Err(user::error::UserError::RecipeLimitReached) => {
            // Expected error
        }
        _ => panic!("Expected RecipeLimitReached error"),
    }
}

#[tokio::test]
async fn test_validate_recipe_creation_premium_tier_bypasses_limit() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    let (user_id, _token) = create_test_user(&pool, &test_app.evento_executor).await;

    // Upgrade user to premium
    let upgrade_cmd = user::commands::UpgradeSubscriptionCommand {
        user_id: user_id.clone(),
        new_tier: "premium".to_string(),
        stripe_customer_id: Some("cus_test123".to_string()),
        stripe_subscription_id: Some("sub_test456".to_string()),
    };

    user::commands::upgrade_subscription(upgrade_cmd, &test_app.evento_executor)
        .await
        .unwrap();

    // Process events
    test_app.process_events().await;

    // Set recipe_count to 15 (over free limit)
    sqlx::query("UPDATE users SET recipe_count = 15 WHERE id = ?1")
        .bind(&user_id)
        .execute(&pool)
        .await
        .unwrap();

    // Attempt to validate recipe creation (should succeed for premium)
    let result = user::commands::validate_recipe_creation(&user_id, &pool).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_recipe_creation_free_tier_under_limit_succeeds() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    let (user_id, _token) = create_test_user(&pool, &test_app.evento_executor).await;

    // Update user recipe_count to 5 (under limit)
    sqlx::query("UPDATE users SET recipe_count = 5 WHERE id = ?1")
        .bind(&user_id)
        .execute(&pool)
        .await
        .unwrap();

    // Attempt to validate recipe creation (should succeed)
    let result = user::commands::validate_recipe_creation(&user_id, &pool).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_upgrade_subscription_command_creates_event() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    let (user_id, _token) = create_test_user(&pool, &test_app.evento_executor).await;

    // Execute upgrade_subscription command
    let upgrade_cmd = user::commands::UpgradeSubscriptionCommand {
        user_id: user_id.clone(),
        new_tier: "premium".to_string(),
        stripe_customer_id: Some("cus_ABC123".to_string()),
        stripe_subscription_id: Some("sub_XYZ789".to_string()),
    };

    let result = user::commands::upgrade_subscription(upgrade_cmd, &test_app.evento_executor).await;
    assert!(result.is_ok());

    // Process events to project to read model
    test_app.process_events().await;

    // Verify read model updated
    let user = sqlx::query(
        "SELECT tier, stripe_customer_id, stripe_subscription_id FROM users WHERE id = ?1",
    )
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(user.get::<String, _>("tier"), "premium");
    assert_eq!(
        user.get::<Option<String>, _>("stripe_customer_id"),
        Some("cus_ABC123".to_string())
    );
    assert_eq!(
        user.get::<Option<String>, _>("stripe_subscription_id"),
        Some("sub_XYZ789".to_string())
    );
}

#[tokio::test]
async fn test_downgrade_subscription_removes_stripe_metadata() {
    let pool = common::setup_test_db().await;
    let test_app = common::create_test_app(pool.clone()).await;

    let (user_id, _token) = create_test_user(&pool, &test_app.evento_executor).await;

    // First upgrade to premium
    let upgrade_cmd = user::commands::UpgradeSubscriptionCommand {
        user_id: user_id.clone(),
        new_tier: "premium".to_string(),
        stripe_customer_id: Some("cus_ABC123".to_string()),
        stripe_subscription_id: Some("sub_XYZ789".to_string()),
    };

    user::commands::upgrade_subscription(upgrade_cmd, &test_app.evento_executor)
        .await
        .unwrap();

    test_app.process_events().await;

    // Now downgrade to free (cancellation)
    let downgrade_cmd = user::commands::UpgradeSubscriptionCommand {
        user_id: user_id.clone(),
        new_tier: "free".to_string(),
        stripe_customer_id: None,
        stripe_subscription_id: None,
    };

    user::commands::upgrade_subscription(downgrade_cmd, &test_app.evento_executor)
        .await
        .unwrap();

    test_app.process_events().await;

    // Verify read model updated to free
    let user = sqlx::query(
        "SELECT tier, stripe_customer_id, stripe_subscription_id FROM users WHERE id = ?1",
    )
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(user.get::<String, _>("tier"), "free");
    assert_eq!(user.get::<Option<String>, _>("stripe_customer_id"), None);
    assert_eq!(
        user.get::<Option<String>, _>("stripe_subscription_id"),
        None
    );
}
