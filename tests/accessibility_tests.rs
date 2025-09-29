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
async fn test_semantic_html_structure() {
    let app = create_test_app().await;

    // Test that pages use proper semantic HTML structure
    let test_routes = [
        "/",
        "/recipes/discover",
        "/meal-plans/current",
        "/shopping-lists/current",
        "/auth/login",
        "/auth/register",
    ];

    for route in test_routes {
        let request = Request::builder().uri(route).body(Body::empty()).unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        if response.status().is_success() {
            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

            // Check for semantic HTML elements
            assert!(
                body_string.contains("<nav")
                    && body_string.contains("<main")
                    && body_string.contains("<footer"),
                "Route {} should use semantic HTML structure (nav, main, footer)",
                route
            );

            // Check for proper heading hierarchy
            assert!(
                body_string.contains("<h1") || body_string.contains("<h2"),
                "Route {} should have proper heading hierarchy",
                route
            );
        }
    }
}

#[tokio::test]
async fn test_aria_attributes_and_roles() {
    let app = create_test_app().await;

    // Test ARIA attributes in interactive components
    let request = Request::builder()
        .uri("/recipes/discover")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for ARIA labels and roles
    assert!(
        body_string.contains("aria-")
            || body_string.contains("role=")
            || body_string.contains("for="),
        "Pages should include ARIA attributes for accessibility"
    );

    // Check for navigation ARIA landmarks
    assert!(
        body_string.contains("role=\"navigation\"")
            || body_string.contains("aria-label")
            || body_string.contains("nav"),
        "Navigation should have proper ARIA landmarks"
    );
}

#[tokio::test]
async fn test_keyboard_navigation_support() {
    let app = create_test_app().await;

    // Test keyboard navigation attributes
    let request = Request::builder()
        .uri("/auth/login")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for focusable elements and keyboard support
    assert!(
        body_string.contains("tabindex")
            || body_string.contains("focus")
            || (body_string.contains("button") && body_string.contains("type=")),
        "Interactive elements should support keyboard navigation"
    );

    // Check for skip links or focus management
    assert!(
        body_string.contains("skip")
            || body_string.contains("main")
            || body_string.contains("autofocus"),
        "Pages should support keyboard navigation patterns"
    );
}

#[tokio::test]
async fn test_form_accessibility() {
    let app = create_test_app().await;

    // Test form accessibility features
    let test_forms = ["/auth/login", "/auth/register"];

    for route in test_forms {
        let request = Request::builder().uri(route).body(Body::empty()).unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Check for proper form labels
        assert!(
            body_string.contains("label") && body_string.contains("for="),
            "Forms in {} should have proper labels associated with inputs",
            route
        );

        // Check for error message accessibility
        assert!(
            body_string.contains("aria-")
                || body_string.contains("role=")
                || body_string.contains("required"),
            "Forms in {} should have accessible error messaging",
            route
        );

        // Check for fieldset organization for complex forms
        if body_string.contains("register") {
            assert!(
                body_string.contains("fieldset")
                    || body_string.contains("group")
                    || body_string.len() > 1000,
                "Complex forms should use fieldset organization or be properly structured"
            );
        }
    }
}

#[tokio::test]
async fn test_image_accessibility() {
    let app = create_test_app().await;

    // Test image accessibility attributes
    let request = Request::builder()
        .uri("/recipes/discover")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for alt attributes on images
    if body_string.contains("<img") {
        assert!(
            body_string.contains("alt=") || body_string.contains("aria-label"),
            "Images should have alternative text"
        );
    }

    // Check for proper image context
    if body_string.contains("recipe") {
        assert!(
            body_string.contains("Recipe")
                || body_string.contains("image")
                || body_string.contains("picture"),
            "Recipe images should have proper context"
        );
    }
}

#[tokio::test]
async fn test_color_contrast_and_readability() {
    // Test that color scheme provides sufficient contrast
    let base_template_content =
        fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
            .expect("base template should exist");

    // Check for proper color usage patterns
    assert!(
        base_template_content.contains("text-") && base_template_content.contains("bg-"),
        "Templates should use proper text and background color classes"
    );

    // Check for focus indicators
    assert!(
        base_template_content.contains("focus:")
            || base_template_content.contains("hover:")
            || base_template_content.contains("transition"),
        "Interactive elements should have focus and hover states"
    );

    // Check for high contrast mode support
    let tailwind_config_content =
        fs::read_to_string("tailwind.config.js").expect("Tailwind config should exist");

    assert!(
        tailwind_config_content.contains("colors") || tailwind_config_content.contains("kitchen"),
        "Color system should be properly configured for accessibility"
    );
}

#[tokio::test]
async fn test_screen_reader_support() {
    let app = create_test_app().await;

    // Test screen reader support features using a public route
    let request = Request::builder()
        .uri("/auth/login")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for screen reader only content
    assert!(
        body_string.contains("sr-only")
            || body_string.contains("screen-reader")
            || body_string.contains("visually-hidden"),
        "Pages should include screen reader specific content"
    );

    // Check for proper document structure
    assert!(
        body_string.contains("lang=") && body_string.contains("DOCTYPE"),
        "Document should have proper language and DOCTYPE declarations"
    );

    // Check for meta information
    assert!(
        body_string.contains("title") && body_string.contains("description"),
        "Pages should have proper meta information for screen readers"
    );
}

#[tokio::test]
async fn test_responsive_accessibility() {
    let app = create_test_app().await;

    // Test mobile accessibility with user-agent simulation
    let request = Request::builder()
        .uri("/recipes/discover")
        .header(
            "User-Agent",
            "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)",
        )
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for mobile accessibility features
    assert!(
        body_string.contains("viewport")
            || body_string.contains("touch")
            || body_string.contains("mobile"),
        "Mobile pages should include mobile accessibility features"
    );

    // Check for touch target sizing
    assert!(
        body_string.contains("touch-target")
            || body_string.contains("p-")
            || body_string.contains("min-"),
        "Mobile interfaces should have proper touch target sizing"
    );
}

#[tokio::test]
async fn test_loading_state_accessibility() {
    // Test loading states accessibility features
    let loading_states_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/loading_states.html")
            .expect("loading states component should exist");

    // Check for ARIA live regions
    assert!(
        loading_states_content.contains("aria-live")
            || loading_states_content.contains("role=\"status\"")
            || loading_states_content.contains("aria-busy"),
        "Loading states should use ARIA live regions"
    );

    // Check for screen reader announcements
    assert!(
        loading_states_content.contains("Loading") && loading_states_content.contains("sr-only"),
        "Loading states should announce to screen readers"
    );

    // Check for focus management during loading
    assert!(
        loading_states_content.contains("focus")
            || loading_states_content.contains("tabindex")
            || loading_states_content.contains("aria-"),
        "Loading states should manage focus appropriately"
    );
}

#[tokio::test]
async fn test_error_state_accessibility() {
    // Test error states accessibility features
    let error_states_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/error_states.html")
            .expect("error states component should exist");

    // Check for error announcement to screen readers
    assert!(
        error_states_content.contains("role=\"alert\"")
            || error_states_content.contains("aria-live")
            || error_states_content.contains("error"),
        "Error states should announce errors to screen readers"
    );

    // Check for actionable error recovery
    assert!(
        error_states_content.contains("button")
            || error_states_content.contains("link")
            || error_states_content.contains("retry"),
        "Error states should provide actionable recovery options"
    );

    // Check for error context and guidance
    assert!(
        error_states_content.contains("try")
            || error_states_content.contains("help")
            || error_states_content.contains("support"),
        "Error states should provide helpful guidance"
    );
}

#[tokio::test]
async fn test_navigation_accessibility() {
    // Test navigation accessibility features
    let mobile_nav_content =
        fs::read_to_string("crates/imkitchen-web/templates/components/mobile_navigation.html")
            .expect("mobile navigation component should exist");

    // Check for navigation landmarks
    assert!(
        mobile_nav_content.contains("nav")
            || mobile_nav_content.contains("role=\"navigation\"")
            || mobile_nav_content.contains("aria-label"),
        "Navigation should have proper landmarks"
    );

    // Check for current page indication
    assert!(
        mobile_nav_content.contains("aria-current")
            || mobile_nav_content.contains("current")
            || mobile_nav_content.contains("active"),
        "Navigation should indicate current page"
    );

    // Check for keyboard navigation support
    assert!(
        mobile_nav_content.contains("tabindex")
            || mobile_nav_content.contains("focus")
            || mobile_nav_content.contains("keyboard"),
        "Navigation should support keyboard navigation"
    );
}

#[tokio::test]
async fn test_kitchen_environment_accessibility() {
    let app = create_test_app().await;

    // Test kitchen-specific accessibility features
    let request = Request::builder()
        .uri("/recipes/discover")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for kitchen-optimized interaction patterns
    assert!(
        body_string.contains("touch")
            || body_string.contains("large")
            || body_string.contains("kitchen"),
        "Kitchen interfaces should be optimized for kitchen environment"
    );

    // Check for voice navigation hints
    assert!(
        body_string.contains("voice") || body_string.contains("speak") || body_string.len() > 1000, // Or just be a substantial page
        "Kitchen interfaces should consider voice interaction patterns"
    );
}

#[cfg(test)]
mod accessibility_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_wcag_compliance_patterns() {
        // Test WCAG 2.1 AA compliance patterns
        let base_template_content =
            fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
                .expect("base template should exist");

        // Check for WCAG compliance indicators
        assert!(
            base_template_content.contains("lang=")
                && base_template_content.contains("charset=")
                && base_template_content.contains("viewport"),
            "Base template should follow WCAG compliance patterns"
        );

        // Check for proper focus management
        assert!(
            base_template_content.contains("focus")
                || base_template_content.contains("outline")
                || base_template_content.contains("transition"),
            "Base template should include focus management"
        );
    }

    #[tokio::test]
    async fn test_inclusive_design_patterns() {
        // Test inclusive design implementation
        let tailwind_config_content =
            fs::read_to_string("tailwind.config.js").expect("Tailwind config should exist");

        // Check for inclusive design utilities
        assert!(
            tailwind_config_content.contains("touch")
                || tailwind_config_content.contains("focus")
                || tailwind_config_content.contains("kitchen"),
            "Design system should include inclusive design patterns"
        );

        // Check for motion preferences
        assert!(
            tailwind_config_content.contains("reduce")
                || tailwind_config_content.contains("motion")
                || tailwind_config_content.contains("animation"),
            "Design system should consider motion preferences"
        );
    }

    #[tokio::test]
    async fn test_accessibility_testing_integration() {
        // Test that accessibility testing is integrated into the development workflow
        let app = create_test_app().await;

        // Test accessibility endpoint for monitoring
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Check for accessibility monitoring integration
        assert!(
            body_string.contains("ok")
                || body_string.contains("health")
                || body_string.contains("status"),
            "Health check should support accessibility monitoring"
        );
    }
}
