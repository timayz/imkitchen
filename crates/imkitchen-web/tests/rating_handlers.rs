use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
    Router,
};
use imkitchen_web::create_app;
// use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_rate_recipe_endpoint() {
    let app = create_test_app().await;
    let recipe_id = Uuid::new_v4();

    // Test valid rating submission
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/recipes/{}/ratings", recipe_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "recipe_id=".to_owned() + &recipe_id.to_string() + "&star_rating=5",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return updated rating component
    assert!(body_str.contains("stars"));
}

#[tokio::test]
async fn test_rate_recipe_endpoint_invalid_rating() {
    let app = create_test_app().await;
    let recipe_id = Uuid::new_v4();

    // Test invalid rating (0 stars)
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/recipes/{}/ratings", recipe_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "recipe_id=".to_owned() + &recipe_id.to_string() + "&star_rating=0",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return error message
    assert!(body_str.contains("Error") || body_str.contains("Invalid"));
}

#[tokio::test]
async fn test_list_reviews_endpoint() {
    let app = create_test_app().await;
    let recipe_id = Uuid::new_v4();

    // Test getting reviews for a recipe
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/recipes/{}/reviews", recipe_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return reviews template
    assert!(body_str.contains("Reviews"));
}

#[tokio::test]
async fn test_list_reviews_with_filters() {
    let app = create_test_app().await;
    let recipe_id = Uuid::new_v4();

    // Test with rating filter
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/recipes/{}/reviews?rating=5&sort=newest",
                    recipe_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test with photos filter
    let app2 = create_test_app().await;
    let response2 = app2
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/recipes/{}/reviews?photos=true", recipe_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_review_endpoint() {
    let app = create_test_app().await;
    let recipe_id = Uuid::new_v4();
    let rating_id = Uuid::new_v4();

    // Test valid review creation
    let form_data = format!(
        "recipe_id={}&rating_id={}&review_text={}",
        recipe_id,
        rating_id,
        "This is an excellent recipe with very detailed instructions and amazing results!"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/reviews")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return success message
    assert!(body_str.contains("success") || body_str.contains("submitted"));
}

#[tokio::test]
async fn test_create_review_validation_error() {
    let app = create_test_app().await;
    let recipe_id = Uuid::new_v4();
    let rating_id = Uuid::new_v4();

    // Test review too short (validation error)
    let form_data = format!(
        "recipe_id={}&rating_id={}&review_text={}",
        recipe_id,
        rating_id,
        "Short" // Less than 10 characters
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/reviews")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return validation error
    assert!(body_str.contains("Error") || body_str.contains("validation"));
}

#[tokio::test]
async fn test_edit_review_endpoint() {
    let app = create_test_app().await;
    let review_id = Uuid::new_v4();

    // Test review editing
    let form_data = "review_text=This is the updated review text with much more detailed information about the recipe.";

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/reviews/{}/edit", review_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return success message
    assert!(body_str.contains("success") || body_str.contains("updated"));
}

#[tokio::test]
async fn test_delete_review_endpoint() {
    let app = create_test_app().await;
    let review_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/reviews/{}", review_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return empty content (review removed)
    assert!(body_str.is_empty());
}

#[tokio::test]
async fn test_update_review_helpfulness_endpoint() {
    let app = create_test_app().await;
    let review_id = Uuid::new_v4();

    // Test marking review as helpful
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/reviews/{}/helpful", review_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("helpful=true"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return updated helpfulness count
    assert!(body_str.contains("helpful") || body_str.contains("people"));
}

#[tokio::test]
async fn test_flag_review_endpoint() {
    let app = create_test_app().await;
    let review_id = Uuid::new_v4();

    // Test flagging a review
    let form_data = "flag_reason=This review contains inappropriate language and should be reviewed by moderators.";

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/reviews/{}/flag", review_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return success message
    assert!(body_str.contains("flagged") || body_str.contains("moderation"));
}

#[tokio::test]
async fn test_flag_review_validation_error() {
    let app = create_test_app().await;
    let review_id = Uuid::new_v4();

    // Test flagging with invalid reason (too short)
    let form_data = "flag_reason=Bad"; // Less than 10 characters

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/reviews/{}/flag", review_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return validation error
    assert!(body_str.contains("Error"));
}

#[tokio::test]
async fn test_rating_distribution_endpoint() {
    let app = create_test_app().await;
    let recipe_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/recipes/{}/rating-distribution", recipe_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return rating distribution template
    assert!(body_str.contains("Rating Distribution") || body_str.contains("avg"));
}

// Admin moderation endpoint tests
#[tokio::test]
async fn test_admin_moderation_panel_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/reviews/moderate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return moderation panel
    assert!(body_str.contains("Moderation") || body_str.contains("pending"));
}

#[tokio::test]
async fn test_admin_moderation_queue_stats_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/reviews/queue")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return statistics dashboard
    assert!(body_str.contains("Pending") && body_str.contains("Approved"));
}

#[tokio::test]
async fn test_admin_approve_review_endpoint() {
    let app = create_test_app().await;
    let review_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/admin/reviews/{}/approve", review_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return approved status
    assert!(body_str.contains("Approved"));
}

#[tokio::test]
async fn test_admin_reject_review_endpoint() {
    let app = create_test_app().await;
    let review_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/admin/reviews/{}/reject", review_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return rejected status
    assert!(body_str.contains("Rejected"));
}

#[tokio::test]
async fn test_admin_bulk_approve_reviews_endpoint() {
    let app = create_test_app().await;

    let form_data = format!(
        "review_ids={},{},{}",
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4()
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/reviews/bulk-approve")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return bulk approval confirmation
    assert!(body_str.contains("approval") || body_str.contains("3"));
}

// Test invalid endpoints and edge cases
#[tokio::test]
async fn test_invalid_recipe_id() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/recipes/invalid-uuid/reviews")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle invalid UUID gracefully
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_invalid_review_id() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/reviews/invalid-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle invalid UUID gracefully
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_missing_required_parameters() {
    let app = create_test_app().await;

    // Test creating review without required fields
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/reviews")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("incomplete=data"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should return validation error or form with errors
    assert!(body_str.contains("Error") || body_str.contains("required"));
}

// Helper function to create test app
async fn create_test_app() -> Router {
    // Create test app configuration
    // In a real test, this would set up a test database and mock services
    create_app()
}

// Integration test for complete rating workflow
#[tokio::test]
async fn test_complete_rating_workflow() {
    let app = create_test_app().await;
    let recipe_id = Uuid::new_v4();

    // Step 1: Submit a rating
    let rating_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/recipes/{}/ratings", recipe_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(format!("recipe_id={}&star_rating=5", recipe_id)))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(rating_response.status(), StatusCode::OK);

    // Step 2: Submit a review for the rating
    let app2 = create_test_app().await;
    let rating_id = Uuid::new_v4(); // In real test, this would come from the rating response
    let review_response = app2
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/reviews")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(format!(
                    "recipe_id={}&rating_id={}&review_text={}",
                    recipe_id,
                    rating_id,
                    "This is an excellent recipe with very clear instructions and amazing results!"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(review_response.status(), StatusCode::OK);

    // Step 3: Check rating distribution
    let app3 = create_test_app().await;
    let distribution_response = app3
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/recipes/{}/rating-distribution", recipe_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(distribution_response.status(), StatusCode::OK);

    // Step 4: List reviews
    let app4 = create_test_app().await;
    let reviews_response = app4
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/recipes/{}/reviews", recipe_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reviews_response.status(), StatusCode::OK);
}
