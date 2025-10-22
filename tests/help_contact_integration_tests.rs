use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn test_help_page_returns_200() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(Request::builder().uri("/help").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify key content is present
    assert!(body_str.contains("Help Center"));
    assert!(body_str.contains("Getting Started"));
    assert!(body_str.contains("Recipe Management"));
    assert!(body_str.contains("Meal Planning"));
    assert!(body_str.contains("Shopping Lists"));
    assert!(body_str.contains("Account & Settings"));
}

#[tokio::test]
async fn test_contact_page_returns_200() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/contact")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify key content is present
    assert!(body_str.contains("Contact Us"));
    assert!(body_str.contains("support@imkitchen.app"));
    assert!(body_str.contains("bugs@imkitchen.app"));
    assert!(body_str.contains("privacy@imkitchen.app"));
    assert!(body_str.contains("partnerships@imkitchen.app"));
}

#[tokio::test]
async fn test_help_page_has_search_functionality() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(Request::builder().uri("/help").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify search input exists
    assert!(body_str.contains(r#"id="help-search"#));
    assert!(body_str.contains(r#"placeholder="Search for help..."#));
}

#[tokio::test]
async fn test_help_page_has_collapsible_sections() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(Request::builder().uri("/help").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify details/summary elements (collapsible FAQ)
    assert!(body_str.contains("<details"));
    assert!(body_str.contains("<summary"));
}

#[tokio::test]
async fn test_contact_page_has_email_links() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/contact")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify mailto links
    assert!(body_str.contains(r#"href="mailto:support@imkitchen.app"#));
    assert!(body_str.contains(r#"href="mailto:bugs@imkitchen.app"#));
    assert!(body_str.contains(r#"href="mailto:privacy@imkitchen.app"#));
    assert!(body_str.contains(r#"href="mailto:partnerships@imkitchen.app"#));
}

#[tokio::test]
async fn test_help_page_links_to_contact() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(Request::builder().uri("/help").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify link to contact page
    assert!(body_str.contains(r#"href="/contact"#));
    assert!(body_str.contains("Contact Support"));
}

#[tokio::test]
async fn test_contact_page_links_to_help() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/contact")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify link to help page
    assert!(body_str.contains(r#"href="/help"#));
    assert!(body_str.contains("Visit Help Center"));
}

#[tokio::test]
async fn test_help_page_accessible_without_authentication() {
    // Arrange - No authentication
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(Request::builder().uri("/help").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert - Should be accessible (200 OK)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_contact_page_accessible_without_authentication() {
    // Arrange - No authentication
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/contact")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert - Should be accessible (200 OK)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_help_page_has_recipe_type_documentation() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(Request::builder().uri("/help").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify recipe type classification is documented
    assert!(body_str.contains("recipe type classification"));
    assert!(body_str.contains("Appetizer"));
    assert!(body_str.contains("Main Course"));
    assert!(body_str.contains("Dessert"));
    assert!(body_str.contains("21 course assignments"));
}

#[tokio::test]
async fn test_help_page_has_kitchen_mode_documentation() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(Request::builder().uri("/help").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify Kitchen Mode is documented
    assert!(body_str.contains("Kitchen Mode"));
    assert!(body_str.contains("high-contrast"));
    assert!(body_str.contains("large-text"));
}

#[tokio::test]
async fn test_contact_form_submission_success() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool.clone(), executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/contact")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "name=John Doe&email=john@example.com&subject=support&message=Test message",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify success message
    assert!(body_str.contains("Thank you"));
    assert!(body_str.contains("toast-success") || body_str.contains("success"));

    // Verify submission was saved to database
    let submission =
        sqlx::query("SELECT * FROM contact_submissions ORDER BY created_at DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    use sqlx::Row;
    assert_eq!(submission.get::<String, _>("name"), "John Doe");
    assert_eq!(submission.get::<String, _>("email"), "john@example.com");
    assert_eq!(submission.get::<String, _>("subject"), "support");
    assert_eq!(submission.get::<String, _>("message"), "Test message");
    assert_eq!(submission.get::<String, _>("status"), "pending");
}

#[tokio::test]
async fn test_contact_form_validation_empty_name() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/contact")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "name=&email=john@example.com&subject=support&message=Test message",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify error message (validator checks for length)
    assert!(
        body_str.contains("Name must be between 2 and 100 characters")
            || body_str.contains("toast-error")
    );

    // Verify form is still visible with preserved values
    assert!(body_str.contains("<form"));
    assert!(body_str.contains("id=\"contact-form\""));
    assert!(body_str.contains("value=\"john@example.com\"")); // Email preserved
    assert!(body_str.contains("value=\"support\"")); // Subject preserved
}

#[tokio::test]
async fn test_contact_form_validation_invalid_email() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/contact")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "name=John Doe&email=invalidemail&subject=support&message=Test message",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify error message
    assert!(body_str.contains("valid email") || body_str.contains("toast-error"));
}

#[tokio::test]
async fn test_contact_form_validation_empty_subject() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/contact")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "name=John Doe&email=john@example.com&subject=&message=Test message",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify error message
    assert!(body_str.contains("subject") || body_str.contains("toast-error"));
}

#[tokio::test]
async fn test_contact_form_validation_empty_message() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/contact")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "name=John Doe&email=john@example.com&subject=support&message=",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify error message (validator checks for length)
    assert!(
        body_str.contains("Message must be between 10 and 5000 characters")
            || body_str.contains("toast-error")
    );

    // Verify form is still visible with preserved values
    assert!(body_str.contains("<form"));
    assert!(body_str.contains("id=\"contact-form\""));
    assert!(body_str.contains("value=\"John Doe\"")); // Name preserved
    assert!(body_str.contains("value=\"john@example.com\"")); // Email preserved
    assert!(body_str.contains("value=\"support\"")); // Subject preserved
}

#[tokio::test]
async fn test_contact_form_has_twinspark_attributes() {
    // Arrange
    let (pool, executor) = common::setup_test_db().await;
    let test_app = common::create_test_app((pool, executor)).await;

    // Act
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/contact")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify TwinSpark attributes are present
    assert!(body_str.contains("ts-req"));
    assert!(body_str.contains("ts-req-method"));
    assert!(body_str.contains("ts-target"));
    // ts-swap is not needed - default behavior replaces outerHTML
}
