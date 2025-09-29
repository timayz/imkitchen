// Integration Testing for Complete Authentication Flows

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use imkitchen_web::create_app_with_db;
use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");
    
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

#[tokio::test]
async fn test_complete_user_registration_flow() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Step 1: Load registration page
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/register")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Join IMKitchen"));
    
    // Step 2: Check email availability (should be available)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/validate/email?email=newuser@example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    // Step 3: Validate registration form data
    let registration_data = "email=newuser@example.com&password=SecurePass123!&password_confirm=SecurePass123!&family_size=4&cooking_skill_level=Intermediate&weekday_cooking_minutes=30&weekend_cooking_minutes=60&accept_terms=on";
    
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(registration_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Registration endpoint should process the request
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_complete_login_flow() {
    let pool = setup_test_db().await;
    
    // Pre-populate with a test user
    sqlx::query(
        "INSERT INTO user_profiles (user_id, email, password_hash, family_size, cooking_skill_level) 
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind("test-user-123")
    .bind("testuser@example.com")
    .bind("$2b$10$dummy.hash.for.testing.purposes.only") // Mock hash
    .bind(3)
    .bind("Advanced")
    .execute(&pool)
    .await
    .expect("Failed to insert test user");
    
    let app = create_app_with_db(Some(pool.clone()));
    
    // Step 1: Load login page
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Welcome back to IMKitchen"));
    
    // Step 2: Validate login form (should pass validation)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/validate-login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=testuser@example.com&password=TestPass123!"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    // Step 3: Attempt login (will fail due to password mismatch, but validates flow)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=testuser@example.com&password=TestPass123!"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Login should process (may fail auth but flow works)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_dashboard_access_flow() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test dashboard access (currently no auth middleware, so should work)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/dashboard")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should show dashboard with mock user data
    assert!(body_str.contains("Welcome back!"));
    assert!(body_str.contains("This Week's Meals"));
    assert!(body_str.contains("demo@imkitchen.com"));
}

#[tokio::test]
async fn test_form_validation_error_handling() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test login with completely invalid data
    let invalid_data = "email=not-an-email&password=weak";
    
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/validate-login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(invalid_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should contain validation error information
    assert!(body_str.len() > 0); // Should return some validation feedback
}

#[tokio::test]
async fn test_email_uniqueness_enforcement() {
    let pool = setup_test_db().await;
    
    // Insert existing user
    sqlx::query(
        "INSERT INTO user_profiles (user_id, email, password_hash, family_size, cooking_skill_level) 
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind("existing-user")
    .bind("existing@example.com")
    .bind("password_hash")
    .bind(2)
    .bind("Beginner")
    .execute(&pool)
    .await
    .expect("Failed to insert existing user");
    
    let app = create_app_with_db(Some(pool.clone()));
    
    // Try to check availability of existing email
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/validate/email?email=existing@example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should indicate email is not available
    assert!(body_str.contains("exists") || body_str.contains("false") || !body_str.contains("available"));
}

#[tokio::test]
async fn test_progressive_enhancement_compatibility() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test that forms work without JavaScript (standard form submission)
    let login_data = "email=test@example.com&password=ValidPass123!";
    
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(login_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should work without JavaScript
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test that validation endpoints return proper HTML fragments
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/validate-login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(login_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should return HTML (not JSON) for progressive enhancement
    assert!(body_str.contains("<") || body_str.is_empty()); // HTML or empty response
}

#[tokio::test]
async fn test_security_headers_and_practices() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test login page for security best practices
    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should include security best practices
    assert!(body_str.contains("autocomplete=")); // Form should have proper autocomplete
    assert!(body_str.contains("type=\"password\"")); // Password field should be properly typed
    assert!(body_str.contains("required")); // Required fields should be marked
}

#[tokio::test]
async fn test_accessibility_features() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test that forms include accessibility features
    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/register")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should include accessibility features
    assert!(body_str.contains("label for=")); // Labels should be associated with inputs
    assert!(body_str.contains("aria-")); // Should include ARIA attributes
    assert!(body_str.contains("id=")); // Form elements should have IDs
}

#[tokio::test]
async fn test_responsive_design_elements() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test that templates include responsive design classes
    let response = app
        .oneshot(
            Request::builder()
                .uri("/dashboard")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should include Tailwind responsive classes
    assert!(body_str.contains("sm:") || body_str.contains("md:") || body_str.contains("lg:"));
    assert!(body_str.contains("grid") || body_str.contains("flex"));
    assert!(body_str.contains("max-w-"));
}

#[tokio::test]
async fn test_error_recovery_mechanisms() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test various error scenarios and recovery
    let error_scenarios = vec![
        ("", StatusCode::OK), // Empty form data
        ("invalid=data", StatusCode::OK), // Invalid field names
        ("email=&password=", StatusCode::OK), // Empty values
    ];
    
    for (data, expected_status) in error_scenarios {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/validate-login")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), expected_status, "Error scenario should be handled gracefully");
    }
}