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
async fn test_minimum_touch_target_sizes() {
    // Read mobile navigation component to validate touch target sizes
    let mobile_nav_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/mobile_navigation.html")
            .expect("mobile navigation template should exist");

    // Check for minimum 44px touch targets (kitchen-optimized)
    assert!(
        mobile_nav_content.contains("h-16") || mobile_nav_content.contains("min-h-[44px]"),
        "mobile navigation should have minimum 44px height for touch targets"
    );

    // Check for touch-manipulation CSS class
    assert!(
        mobile_nav_content.contains("touch-manipulation"),
        "mobile navigation should use touch-manipulation for better touch response"
    );

    // Check for adequate spacing between touch targets
    assert!(
        mobile_nav_content.contains("space-y-1") || mobile_nav_content.contains("gap-"),
        "mobile navigation should have adequate spacing between touch targets"
    );
}

#[tokio::test]
async fn test_form_components_touch_friendly() {
    let app = create_test_app().await;

    // Test login form renders with touch-friendly components
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

    // Check for touch-friendly input sizes
    assert!(
        body_string.contains("py-3")
            || body_string.contains("py-4")
            || body_string.contains("h-12"),
        "login form should have touch-friendly input heights"
    );

    // Check for touch-friendly button sizes
    assert!(
        body_string.contains("px-4")
            && (body_string.contains("py-3") || body_string.contains("py-4")),
        "login form should have touch-friendly button sizes"
    );

    // Check for adequate spacing between form elements
    assert!(
        body_string.contains("space-y-8")
            || body_string.contains("space-y-6")
            || body_string.contains("space-y-4")
            || body_string.contains("mb-"),
        "login form should have adequate spacing between elements"
    );
}

#[tokio::test]
async fn test_tailwind_touch_configuration() {
    // Read Tailwind configuration to validate touch optimizations
    let base_template = fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
        .expect("base.html template should exist");

    // Check for kitchen-optimized touch targets in Tailwind config
    assert!(
        base_template.contains("tailwind.config"),
        "base template should contain Tailwind configuration"
    );

    // Check for kitchen color theme (indicates custom configuration)
    assert!(
        base_template.contains("kitchen"),
        "base template should use kitchen color theme for touch interfaces"
    );
}

#[tokio::test]
async fn test_haptic_feedback_simulation() {
    // Read templates to validate CSS transitions for haptic feedback simulation
    let mobile_nav_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/mobile_navigation.html")
            .expect("mobile navigation template should exist");

    // Check for transition classes that simulate haptic feedback
    assert!(
        mobile_nav_content.contains("transition-colors")
            || mobile_nav_content.contains("transition"),
        "mobile navigation should have transitions for haptic feedback simulation"
    );

    // Check for active states that provide visual feedback
    assert!(
        mobile_nav_content.contains("active:") || mobile_nav_content.contains("hover:"),
        "mobile navigation should have active/hover states for touch feedback"
    );

    // Check for focus states for accessibility
    assert!(
        mobile_nav_content.contains("focus:"),
        "mobile navigation should have focus states for keyboard navigation"
    );
}

#[tokio::test]
async fn test_touch_gesture_support() {
    let app = create_test_app().await;

    // Test that pages load successfully for touch interaction testing
    let request = Request::builder()
        .uri("/auth/register")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for scroll-friendly containers
    assert!(
        body_string.contains("overflow-") || body_string.contains("scroll"),
        "register page should handle scrolling for long forms on mobile"
    );
}

#[tokio::test]
async fn test_kitchen_environment_optimization() {
    // Read templates to validate kitchen environment optimizations
    let base_template = fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
        .expect("base.html template should exist");

    // Check for user-scalable=no (prevents accidental zoom in kitchen)
    assert!(
        base_template.contains("user-scalable=no"),
        "base template should disable user scaling for kitchen environment"
    );

    // Check for tap-highlight configuration
    assert!(
        base_template.contains("msapplication-tap-highlight"),
        "base template should configure tap highlight for touch interactions"
    );
}

#[tokio::test]
async fn test_progressive_enhancement_touch() {
    // Validate that touch enhancements work with TwinSpark progressive enhancement
    let mobile_nav_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/mobile_navigation.html")
            .expect("mobile navigation template should exist");

    // Check for progressive enhancement patterns
    assert!(
        mobile_nav_content.contains("addEventListener")
            || mobile_nav_content.contains("DOMContentLoaded"),
        "mobile navigation should use progressive enhancement for touch interactions"
    );

    // Check that core functionality works without JavaScript
    assert!(
        mobile_nav_content.contains("href="),
        "mobile navigation should work without JavaScript (progressive enhancement)"
    );
}

#[cfg(test)]
mod touch_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_form_input_touch_optimization() {
        let app = create_test_app().await;

        // Test that form inputs are optimized for touch
        let request = Request::builder()
            .uri("/auth/login")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Check for proper input types for mobile keyboards
        assert!(
            body_string.contains("type=\"email\"") || body_string.contains("type=\"password\""),
            "login form should use proper input types for mobile keyboards"
        );

        // Check for autocomplete attributes for better UX
        assert!(
            body_string.contains("autocomplete="),
            "login form should use autocomplete for better touch UX"
        );
    }

    #[tokio::test]
    async fn test_button_touch_targets() {
        let app = create_test_app().await;

        let request = Request::builder()
            .uri("/auth/register")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Check for touch-friendly button styling
        assert!(
            body_string.contains("px-4") || body_string.contains("px-6"),
            "register form should have adequate horizontal padding for touch targets"
        );
        assert!(
            body_string.contains("py-2")
                || body_string.contains("py-3")
                || body_string.contains("py-4"),
            "register form should have adequate vertical padding for touch targets"
        );
    }
}
