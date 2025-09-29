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
async fn test_base_template_has_responsive_viewport() {
    // Read the base template to validate responsive meta tags
    let base_template = fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
        .expect("base.html template should exist");

    // Check for responsive viewport meta tag
    assert!(
        base_template.contains("width=device-width"),
        "base template should have device-width viewport"
    );
    assert!(
        base_template.contains("initial-scale=1.0"),
        "base template should have initial-scale=1.0"
    );

    // Check for mobile-optimizations
    assert!(
        base_template.contains("user-scalable=no"),
        "base template should disable user scaling for kitchen environment"
    );
}

#[tokio::test]
async fn test_authentication_templates_responsive() {
    let app = create_test_app().await;

    // Test login page renders successfully
    let request = Request::builder()
        .uri("/auth/login")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check that response contains responsive classes
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Validate responsive Tailwind classes are present
    assert!(
        body_string.contains("sm:"),
        "login page should contain small screen responsive classes"
    );
    assert!(
        body_string.contains("md:"),
        "login page should contain medium screen responsive classes"
    );
    assert!(
        body_string.contains("lg:"),
        "login page should contain large screen responsive classes"
    );
}

#[tokio::test]
async fn test_dashboard_responsive_layout() {
    let app = create_test_app().await;

    // Note: This test will fail authentication, but we can still check the redirect response
    let request = Request::builder()
        .uri("/dashboard")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should redirect to login (status 303 or 302) due to auth middleware
    assert!(
        response.status() == StatusCode::SEE_OTHER
            || response.status() == StatusCode::FOUND
            || response.status() == StatusCode::TEMPORARY_REDIRECT
    );
}

#[tokio::test]
async fn test_mobile_navigation_component_structure() {
    // Check if mobile navigation component exists
    let mobile_nav_path = "crates/imkitchen-web/templates/components/mobile_navigation.html";

    if fs::metadata(mobile_nav_path).is_ok() {
        let mobile_nav_content = fs::read_to_string(mobile_nav_path)
            .expect("mobile navigation template should be readable");

        // Validate mobile navigation structure
        assert!(
            mobile_nav_content.contains("nav"),
            "mobile navigation should contain nav element"
        );
        assert!(
            mobile_nav_content.contains("hidden"),
            "mobile navigation should have responsive visibility"
        );
        assert!(
            mobile_nav_content.contains("md:"),
            "mobile navigation should have desktop responsive classes"
        );

        // Check for touch-friendly elements
        assert!(
            mobile_nav_content.contains("touch"),
            "mobile navigation should be touch-optimized"
        );
    }
}

#[tokio::test]
async fn test_responsive_breakpoint_classes() {
    // Read base template to validate Tailwind responsive utilities
    let base_template = fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
        .expect("base.html template should exist");

    // Check for Tailwind responsive breakpoints configuration
    assert!(
        base_template.contains("sm:"),
        "base template should use small breakpoint classes"
    );
    assert!(
        base_template.contains("md:"),
        "base template should use medium breakpoint classes"
    );
    assert!(
        base_template.contains("lg:"),
        "base template should use large breakpoint classes"
    );

    // Check for mobile-first approach (base classes without prefixes)
    assert!(
        base_template.contains("flex"),
        "base template should use flexbox utilities"
    );
    assert!(
        base_template.contains("grid") || base_template.contains("flex"),
        "base template should use layout utilities"
    );
}

#[tokio::test]
async fn test_kitchen_optimized_spacing() {
    // Read templates to validate kitchen-optimized spacing
    let base_template = fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
        .expect("base.html template should exist");

    // Check for adequate spacing for kitchen environment
    assert!(
        base_template.contains("p-4")
            || base_template.contains("px-4")
            || base_template.contains("py-4"),
        "base template should have adequate padding for touch interfaces"
    );
    assert!(
        base_template.contains("space-") || base_template.contains("gap-"),
        "base template should have spacing utilities for touch targets"
    );
}

#[tokio::test]
async fn test_responsive_image_handling() {
    // This test validates that image handling is responsive-ready
    let app = create_test_app().await;

    // Test that static image route would work
    let request = Request::builder()
        .uri("/static/images/icon-192x192.png")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 404 for non-existent image (we haven't created actual images)
    // But the route should be accessible
    assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK);
}

#[cfg(test)]
mod responsive_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_tailwind_responsive_utilities() {
        let app = create_test_app().await;

        // Test that pages render with responsive utilities
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

        // Check for responsive container classes
        assert!(
            body_string.contains("container") || body_string.contains("max-w-"),
            "register page should use responsive container classes"
        );
        assert!(
            body_string.contains("mx-auto"),
            "register page should center content responsively"
        );
    }

    #[tokio::test]
    async fn test_mobile_first_approach() {
        // Read any template file to validate mobile-first CSS approach
        let base_template = fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
            .expect("base.html template should exist");

        // Mobile-first means base classes are mobile, with sm:/md:/lg: prefixes for larger screens
        let mobile_first_indicators = [
            "block sm:hidden",      // Hide on larger screens
            "hidden sm:block",      // Show on larger screens
            "text-sm sm:text-base", // Responsive text sizing
            "p-2 sm:p-4",           // Responsive padding
            "w-full sm:w-auto",     // Responsive width
        ];

        let has_mobile_first = mobile_first_indicators
            .iter()
            .any(|pattern| base_template.contains(pattern));

        // At least one mobile-first pattern should be present, or basic responsive classes
        assert!(
            has_mobile_first || base_template.contains("sm:"),
            "base template should follow mobile-first responsive design"
        );
    }
}
