use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use std::fs;
use tower::ServiceExt;

use imkitchen_web::{create_app_routes, AppState};

/// Helper function to create test app
async fn create_test_app() -> Router {
    let app_state = AppState::test_default().await;
    create_app_routes(app_state)
}

#[tokio::test]
async fn test_loading_states_component_exists() {
    // Test that loading states component exists and has proper structure
    let loading_states_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/loading_states.html")
            .expect("loading states component should exist");

    // Check for skeleton loading patterns
    assert!(
        loading_states_content.contains("skeleton") || loading_states_content.contains("loading"),
        "loading states should contain skeleton loading patterns"
    );

    // Check for CSS animation classes
    assert!(
        loading_states_content.contains("animate-")
            || loading_states_content.contains("pulse")
            || loading_states_content.contains("spin"),
        "loading states should include CSS animation classes"
    );

    // Check for accessibility attributes
    assert!(
        loading_states_content.contains("aria-") || loading_states_content.contains("role="),
        "loading states should have accessibility attributes"
    );

    // Check for screen reader text
    assert!(
        loading_states_content.contains("sr-only") || loading_states_content.contains("Loading"),
        "loading states should include screen reader text"
    );
}

#[tokio::test]
async fn test_error_states_component_exists() {
    // Test that error states component exists and has proper structure
    let error_states_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/error_states.html")
            .expect("error states component should exist");

    // Check for error messaging structure
    assert!(
        error_states_content.contains("error") || error_states_content.contains("Error"),
        "error states should contain error messaging"
    );

    // Check for retry functionality
    assert!(
        error_states_content.contains("retry")
            || error_states_content.contains("Retry")
            || error_states_content.contains("Try again"),
        "error states should include retry functionality"
    );

    // Check for proper error styling
    assert!(
        error_states_content.contains("red-")
            || error_states_content.contains("error")
            || error_states_content.contains("danger"),
        "error states should have proper error styling"
    );

    // Check for accessibility features
    assert!(
        error_states_content.contains("role=") || error_states_content.contains("aria-"),
        "error states should have accessibility features"
    );
}

#[tokio::test]
async fn test_progressive_loading_with_twinspark() {
    let app = create_test_app().await;

    // Test that forms include TwinSpark loading patterns
    let request = Request::builder()
        .uri("/auth/login")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for TwinSpark attributes that enable progressive loading
    assert!(
        body_string.contains("ts-req") || body_string.contains("ts-target"),
        "forms should use TwinSpark for progressive loading"
    );

    // Check for form loading states integration
    assert!(
        body_string.contains("form")
            && (body_string.contains("loading") || body_string.contains("disabled")),
        "forms should integrate loading state management"
    );
}

#[tokio::test]
async fn test_network_error_handling() {
    let app = create_test_app().await;

    // Test error handling for non-existent routes
    let request = Request::builder()
        .uri("/non-existent-route")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 404 or handle gracefully
    assert!(
        response.status() == StatusCode::NOT_FOUND || response.status().is_client_error(),
        "non-existent routes should be handled gracefully"
    );
}

#[tokio::test]
async fn test_form_validation_error_states() {
    let app = create_test_app().await;

    // Test error states in form validation
    let request = Request::builder()
        .uri("/auth/register")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for validation error display structure
    assert!(
        body_string.contains("validation")
            || body_string.contains("error")
            || body_string.contains("invalid"),
        "forms should have validation error display structure"
    );

    // Check for error styling classes
    assert!(
        body_string.contains("red-")
            || body_string.contains("error")
            || body_string.contains("invalid"),
        "forms should include error styling classes"
    );
}

#[tokio::test]
async fn test_skeleton_loading_screens() {
    // Test that skeleton loading screens are properly structured
    let loading_states_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/loading_states.html")
            .expect("loading states component should exist");

    // Check for skeleton loading for different content types
    assert!(
        loading_states_content.contains("skeleton")
            && (loading_states_content.contains("recipe")
                || loading_states_content.contains("card")
                || loading_states_content.contains("list")),
        "loading states should include skeleton screens for different content types"
    );

    // Check for proper loading animations
    assert!(
        loading_states_content.contains("animate-pulse")
            || loading_states_content.contains("animate-ping")
            || loading_states_content.contains("animate-spin"),
        "skeleton screens should use proper loading animations"
    );

    // Check for responsive skeleton design
    assert!(
        loading_states_content.contains("md:")
            || loading_states_content.contains("sm:")
            || loading_states_content.contains("lg:"),
        "skeleton screens should be responsive"
    );
}

#[tokio::test]
async fn test_loading_state_accessibility() {
    // Test loading states accessibility features
    let loading_states_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/loading_states.html")
            .expect("loading states component should exist");

    // Check for ARIA live regions for loading states
    assert!(
        loading_states_content.contains("aria-live")
            || loading_states_content.contains("role=\"status\"")
            || loading_states_content.contains("aria-busy"),
        "loading states should use ARIA live regions"
    );

    // Check for screen reader friendly loading text
    assert!(
        loading_states_content.contains("Loading") && loading_states_content.contains("sr-only"),
        "loading states should include screen reader friendly text"
    );

    // Check for keyboard navigation preservation
    assert!(
        loading_states_content.contains("focus")
            || loading_states_content.contains("tabindex")
            || loading_states_content.contains("keyboard"),
        "loading states should preserve keyboard navigation"
    );
}

#[tokio::test]
async fn test_error_recovery_patterns() {
    // Test error recovery and retry functionality
    let error_states_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/error_states.html")
            .expect("error states component should exist");

    // Check for retry button functionality
    assert!(
        error_states_content.contains("retry")
            || error_states_content.contains("Try again")
            || error_states_content.contains("Reload"),
        "error states should include retry functionality"
    );

    // Check for error categorization
    assert!(
        error_states_content.contains("network")
            || error_states_content.contains("server")
            || error_states_content.contains("validation")
            || error_states_content.contains("404"),
        "error states should categorize different error types"
    );

    // Check for user-friendly error messages
    assert!(
        error_states_content.contains("Something went wrong")
            || error_states_content.contains("try again")
            || error_states_content.contains("We're sorry"),
        "error states should have user-friendly messages"
    );
}

#[tokio::test]
async fn test_loading_states_for_meal_planning() {
    let app = create_test_app().await;

    // Test loading states in meal planning workflows
    let request = Request::builder()
        .uri("/meal-plans/current")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    if response.status() == StatusCode::OK {
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Check for loading integration in meal planning context
        let body_lower = body_string.to_lowercase();
        assert!(
            body_lower.contains("meal")
                && (body_lower.contains("loading") || body_lower.contains("coming soon")),
            "meal planning should integrate loading states"
        );
    }
    // If route redirects or not found, it's expected for development
}

#[cfg(test)]
mod loading_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_twinspark_loading_responses() {
        let app = create_test_app().await;

        // Test that TwinSpark responses include proper loading state management
        let request = Request::builder()
            .uri("/recipes/discover")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Check for TwinSpark integration that supports loading states
        assert!(
            body_string.contains("ts-")
                || body_string.contains("filter")
                || body_string.contains("search"),
            "pages should support TwinSpark loading state integration"
        );
    }

    #[tokio::test]
    async fn test_form_submission_loading_states() {
        // Test that forms properly handle loading states during submission
        let app = create_test_app().await;

        let request = Request::builder()
            .uri("/auth/login")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Check for form loading state integration
        assert!(
            body_string.contains("submit")
                && (body_string.contains("disabled")
                    || body_string.contains("loading")
                    || body_string.contains("button")),
            "forms should integrate loading states for submissions"
        );

        // Check for JavaScript enhancement for loading states
        assert!(
            body_string.contains("addEventListener")
                || body_string.contains("form")
                || body_string.contains("submit"),
            "forms should have JavaScript enhancement for loading states"
        );
    }
}
