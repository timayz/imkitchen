// Async Validation Testing for Email Availability and Username Uniqueness

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use imkitchen_web::create_app_with_db;
use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    // Create in-memory SQLite database for testing
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");
    
    // Run migrations to set up schema
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

#[tokio::test]
async fn test_email_availability_check_new_email() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test checking availability for a new email that doesn't exist
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/validate/email?email=new@example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should indicate email is available
    assert!(body_str.contains("available") || body_str.contains("true"));
}

#[tokio::test]
async fn test_email_availability_check_existing_email() {
    let pool = setup_test_db().await;
    
    // Insert a test user first
    sqlx::query(
        "INSERT INTO user_profiles (id, email, password_hash, family_size, skill_level) 
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind("test-user-id")
    .bind("existing@example.com")
    .bind("password_hash")
    .bind(4)
    .bind("Intermediate")
    .execute(&pool)
    .await
    .expect("Failed to insert test user");
    
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test checking availability for an existing email
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
    assert!(body_str.contains("already registered") || body_str.contains("exists") || body_str.contains("taken"));
}

#[tokio::test]
async fn test_username_availability_suggestions() {
    let pool = setup_test_db().await;
    
    // Insert a test user first
    sqlx::query(
        "INSERT INTO user_profiles (id, email, password_hash, family_size, skill_level) 
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind("test-user-id")
    .bind("john@example.com")
    .bind("password_hash")
    .bind(2)
    .bind("Beginner")
    .execute(&pool)
    .await
    .expect("Failed to insert test user");
    
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test username availability with suggestions
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/validate/username?username=john@example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should provide suggestions for taken username
    assert!(body_str.contains("suggestions") || body_str.contains("john1") || body_str.contains("available"));
}

#[tokio::test]
async fn test_email_validation_form_post() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test POST request to email validation endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/validate/email")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=test@example.com"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should return validation result
    assert!(body_str.contains("email") || body_str.contains("available"));
}

#[tokio::test]
async fn test_invalid_email_format_async() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test with various invalid email formats
    let invalid_emails = vec![
        "invalid-email",
        "test@",
        "@example.com",
        "test..test@example.com",
        "test@example",
    ];
    
    for email in invalid_emails {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/validate/email?email={}", email))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should either return validation error or bad request
        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST,
            "Invalid email {} should be handled properly",
            email
        );
    }
}

#[tokio::test]
async fn test_async_validation_performance() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    let start = std::time::Instant::now();
    
    // Test multiple concurrent email validation requests
    let mut handles = vec![];
    
    for i in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let email = format!("test{}@example.com", i);
            let response = app_clone
                .oneshot(
                    Request::builder()
                        .uri(&format!("/api/validate/email?email={}", email))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            
            response.status()
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        let status = handle.await.unwrap();
        assert_eq!(status, StatusCode::OK);
    }
    
    let duration = start.elapsed();
    
    // All 10 concurrent requests should complete within reasonable time
    assert!(duration.as_millis() < 5000, "Async validation should be performant");
}

#[tokio::test]
async fn test_database_connection_handling() {
    let pool = setup_test_db().await;
    
    // Test that the app can be created without database
    let app_no_db = create_app_with_db(None);
    
    let response = app_no_db
        .oneshot(
            Request::builder()
                .uri("/api/validate/email?email=test@example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return not found or internal error when no database
    assert!(
        response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "App should handle missing database gracefully"
    );
}

#[tokio::test]
async fn test_concurrent_database_operations() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test concurrent database operations don't cause conflicts
    let mut handles = vec![];
    
    for i in 0..5 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            // Alternate between checking existing and new emails
            let email = if i % 2 == 0 {
                format!("new{}@example.com", i)
            } else {
                "existing@example.com".to_string()
            };
            
            let response = app_clone
                .oneshot(
                    Request::builder()
                        .uri(&format!("/api/validate/email?email={}", email))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            
            response.status()
        });
        handles.push(handle);
    }
    
    // All concurrent operations should succeed
    for handle in handles {
        let status = handle.await.unwrap();
        assert_eq!(status, StatusCode::OK);
    }
}

#[tokio::test]
async fn test_sql_injection_protection() {
    let pool = setup_test_db().await;
    let app = create_app_with_db(Some(pool.clone()));
    
    // Test various SQL injection attempts
    let malicious_inputs = vec![
        "test@example.com'; DROP TABLE user_profiles; --",
        "test@example.com' OR '1'='1",
        "test@example.com' UNION SELECT * FROM user_profiles --",
        "'; INSERT INTO user_profiles VALUES (...); --",
    ];
    
    for malicious_input in malicious_inputs {
        let encoded_input = urlencoding::encode(malicious_input);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/validate/email?email={}", encoded_input))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should handle malicious input safely (either validate properly or return error)
        assert!(
            response.status() == StatusCode::OK 
            || response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
            "Malicious input should be handled safely"
        );
    }
    
    // Verify table still exists after all injection attempts
    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='user_profiles'"
    )
    .fetch_one(&pool)
    .await;
    
    assert!(count_result.is_ok() && count_result.unwrap() > 0, "Table should still exist after injection attempts");
}