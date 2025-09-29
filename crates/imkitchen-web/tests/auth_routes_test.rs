// Integration tests for auth routes

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;

use imkitchen_web::create_app;

#[tokio::test]
async fn test_login_form_route() {
    let app: Router = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/login")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Check that it contains expected HTML elements
    assert!(body_str.contains("Welcome back to IMKitchen") || body_str.contains("Sign in to access"));
    assert!(body_str.contains("ts-req=\"/auth/login\""));
    assert!(body_str.contains("ts-req") || body_str.contains("form"));
}

#[tokio::test]
async fn test_login_validation_route() {
    let app: Router = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/login")
                .method("POST")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=invalid&password=short"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should return validation errors in HTML format
    assert!(body_str.contains("text-red-600") || body_str.contains("error"));
}

#[tokio::test]
async fn test_login_post_route() {
    let app: Router = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/login")
                .method("POST")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=test@example.com&password=ValidPass123!"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}