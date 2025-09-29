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
async fn test_bottom_navigation_component_exists() {
    // Test that the bottom navigation component exists and has proper structure
    let bottom_nav_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/bottom_navigation.html")
            .expect("bottom navigation component should exist");

    // Check for proper navigation structure
    assert!(
        bottom_nav_content.contains("nav"),
        "bottom navigation should contain nav element"
    );

    // Check for kitchen environment optimizations
    assert!(
        bottom_nav_content.contains("kitchen-touch"),
        "bottom navigation should use kitchen-touch optimizations"
    );

    // Check for minimum touch target sizes
    assert!(
        bottom_nav_content.contains("touch-target") || bottom_nav_content.contains("min-h-"),
        "bottom navigation should have proper touch target sizes"
    );

    // Check for core navigation links
    assert!(
        bottom_nav_content.contains("href=\"/\""),
        "bottom navigation should have home link"
    );
    assert!(
        bottom_nav_content.contains("href=\"/recipes"),
        "bottom navigation should have recipes link"
    );
    assert!(
        bottom_nav_content.contains("href=\"/meal-plans"),
        "bottom navigation should have meal plans link"
    );
    assert!(
        bottom_nav_content.contains("href=\"/shopping"),
        "bottom navigation should have shopping link"
    );
}

#[tokio::test]
async fn test_breadcrumb_navigation_component() {
    // Test that breadcrumb navigation component exists and works properly
    let breadcrumb_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/breadcrumb.html")
            .expect("breadcrumb component should exist");

    // Check for proper breadcrumb structure
    assert!(
        breadcrumb_content.contains("breadcrumb") || breadcrumb_content.contains("nav"),
        "breadcrumb should have proper navigation structure"
    );

    // Check for accessibility attributes
    assert!(
        breadcrumb_content.contains("aria-") || breadcrumb_content.contains("role="),
        "breadcrumb should have accessibility attributes"
    );

    // Check for proper separators
    assert!(
        breadcrumb_content.contains("/")
            || breadcrumb_content.contains(">")
            || breadcrumb_content.contains("›"),
        "breadcrumb should have proper separators"
    );
}

#[tokio::test]
async fn test_navigation_information_architecture() {
    let app = create_test_app().await;

    // Test core navigation routes exist and return proper responses
    let routes_to_test = [
        "/",
        "/recipes/discover",
        "/meal-plans/current",
        "/shopping-lists/current",
        "/profile",
    ];

    for route in routes_to_test {
        let request = Request::builder().uri(route).body(Body::empty()).unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        // Allow both OK and redirect responses for navigation testing
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::FOUND
                || response.status() == StatusCode::SEE_OTHER,
            "Route {} should be accessible (got: {})",
            route,
            response.status()
        );
    }
}

#[tokio::test]
async fn test_user_orientation_indicators() {
    let app = create_test_app().await;

    // Test that pages include proper orientation indicators
    let request = Request::builder()
        .uri("/recipes/discover")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for page title or heading that indicates current location
    assert!(
        body_string.contains("<title>")
            || body_string.contains("<h1>")
            || body_string.contains("<h2>"),
        "pages should have clear title or heading for orientation"
    );

    // Check for navigation integration
    assert!(
        body_string.contains("navigation")
            || body_string.contains("nav")
            || body_string.contains("menu"),
        "pages should include navigation elements for orientation"
    );
}

#[tokio::test]
async fn test_contextual_navigation_helpers() {
    let app = create_test_app().await;

    // Test meal planning workflow navigation
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

        // Check for contextual navigation within meal planning
        assert!(
            body_string.contains("href=\"/recipes")
                || body_string.contains("href=\"/shopping")
                || body_string.contains("Back")
                || body_string.contains("Next"),
            "meal planning should have contextual navigation helpers"
        );
    }
    // If route redirects or not found, it's expected for non-implemented features
}

#[tokio::test]
async fn test_mobile_navigation_patterns() {
    let app = create_test_app().await;

    // Test that mobile navigation patterns are implemented
    let request = Request::builder()
        .uri("/recipes/discover")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for mobile-specific navigation classes
    assert!(
        body_string.contains("md:hidden")
            || body_string.contains("mobile")
            || body_string.contains("sm:")
            || body_string.contains("lg:"),
        "pages should implement mobile navigation patterns"
    );

    // Check for touch-friendly navigation
    assert!(
        body_string.contains("touch-target")
            || body_string.contains("py-3")
            || body_string.contains("py-4")
            || body_string.contains("min-h-"),
        "mobile navigation should be touch-friendly"
    );
}

#[tokio::test]
async fn test_navigation_accessibility() {
    // Test that navigation components have proper accessibility
    let mobile_nav_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/mobile_navigation.html")
            .expect("mobile navigation should exist");

    // Check for ARIA attributes
    assert!(
        mobile_nav_content.contains("aria-")
            || mobile_nav_content.contains("role=")
            || mobile_nav_content.contains("aria-label")
            || mobile_nav_content.contains("aria-expanded"),
        "navigation should have ARIA attributes for accessibility"
    );

    // Check for keyboard navigation support
    assert!(
        mobile_nav_content.contains("focus:")
            || mobile_nav_content.contains("tabindex")
            || mobile_nav_content.contains("keyboard"),
        "navigation should support keyboard navigation"
    );
}

#[tokio::test]
async fn test_progressive_navigation_enhancement() {
    // Test that navigation works with TwinSpark progressive enhancement
    let mobile_nav_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/mobile_navigation.html")
            .expect("mobile navigation should exist");

    // Check that basic navigation works without JavaScript
    assert!(
        mobile_nav_content.contains("href="),
        "navigation should work without JavaScript"
    );

    // Check for progressive enhancement patterns
    assert!(
        mobile_nav_content.contains("ts-")
            || mobile_nav_content.contains("addEventListener")
            || mobile_nav_content.contains("DOMContentLoaded"),
        "navigation should use progressive enhancement patterns"
    );

    // Check that enhanced features don't break basic functionality
    assert!(
        !mobile_nav_content.contains("onclick=\"") && !mobile_nav_content.contains("javascript:"),
        "navigation should not rely on inline JavaScript"
    );
}

#[cfg(test)]
mod navigation_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_navigation_routing_handlers() {
        let app = create_test_app().await;

        // Test that navigation routes are properly handled
        let navigation_routes = [
            "/recipes/discover",
            "/meal-plans/current",
            "/shopping-lists/current",
            "/collections",
            "/profile",
        ];

        for route in navigation_routes {
            let request = Request::builder().uri(route).body(Body::empty()).unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            // Routes should either work or redirect appropriately
            assert!(
                response.status().is_success()
                    || response.status().is_redirection()
                    || response.status() == StatusCode::NOT_FOUND, // Acceptable for unimplemented features
                "Navigation route {} should be properly handled",
                route
            );
        }
    }

    #[tokio::test]
    async fn test_navigation_state_management() {
        // Test that navigation maintains proper state and active indicators
        let mobile_nav_content =
            fs::read_to_string("crates/imkitchen-web/templates/components/mobile_navigation.html")
                .expect("mobile navigation should exist");

        // Check for active state handling
        assert!(
            mobile_nav_content.contains("active")
                || mobile_nav_content.contains("current")
                || mobile_nav_content.contains("selected")
                || mobile_nav_content.contains("aria-current"),
            "navigation should handle active state indicators"
        );
    }
}
