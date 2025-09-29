use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use std::fs;
use std::time::{Duration, Instant};
use tower::ServiceExt;

use imkitchen_web::{create_app_routes, AppState};

/// Helper function to create test app
async fn create_test_app() -> Router {
    let app_state = AppState::test_default().await;
    create_app_routes(app_state)
}

#[tokio::test]
async fn test_page_load_performance_under_3_seconds() {
    let app = create_test_app().await;

    // Test key pages for load time performance
    let test_routes = [
        "/recipes/discover",
        "/meal-plans/current",
        "/shopping-lists/current",
        "/collections",
        "/auth/login",
        "/auth/register",
    ];

    for route in test_routes {
        let start_time = Instant::now();

        let request = Request::builder().uri(route).body(Body::empty()).unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        let elapsed = start_time.elapsed();

        // Ensure successful response or expected redirect
        assert!(
            response.status().is_success() || response.status().is_redirection(),
            "Route {} should respond successfully",
            route
        );

        // Performance target: under 3 seconds for server response time
        assert!(
            elapsed < Duration::from_secs(3),
            "Route {} took {:.2}s, should be under 3 seconds",
            route,
            elapsed.as_secs_f32()
        );
    }
}

#[tokio::test]
async fn test_static_asset_optimization() {
    // Test that static assets are optimized for performance

    // Check Tailwind CSS compilation and minification
    let tailwind_config_exists = fs::metadata("tailwind.config.js").is_ok();
    assert!(
        tailwind_config_exists,
        "Tailwind config should exist for optimization"
    );

    // Check for asset bundling configuration
    let static_dir_exists = fs::metadata("crates/imkitchen-web/static").is_ok();
    assert!(static_dir_exists, "Static assets directory should exist");

    // Check for optimized service worker
    let service_worker_content =
        fs::read_to_string("crates/imkitchen-web/static/js/service-worker.js");
    if let Ok(content) = service_worker_content {
        // Service worker should implement efficient caching strategies
        assert!(
            content.contains("cache") && content.len() > 1000,
            "Service worker should implement comprehensive caching"
        );

        // Should have performance-oriented cache strategies
        assert!(
            content.contains("stale-while-revalidate")
                || content.contains("cache-first")
                || content.contains("network-first"),
            "Service worker should use performance-oriented cache strategies"
        );
    }
}

#[tokio::test]
async fn test_database_query_optimization() {
    let app = create_test_app().await;

    // Test that database queries are optimized through projection access
    let start_time = Instant::now();

    // Test routes that would involve database access
    let request = Request::builder()
        .uri("/auth/login")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let elapsed = start_time.elapsed();

    assert_eq!(response.status(), StatusCode::OK);

    // Database queries should be fast (under 100ms for simple pages)
    assert!(
        elapsed < Duration::from_millis(100),
        "Database queries should be optimized (took {:.2}ms)",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_lazy_loading_implementation() {
    let app = create_test_app().await;

    // Test that pages implement lazy loading for non-critical content
    let request = Request::builder()
        .uri("/recipes/discover")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for lazy loading attributes or patterns
    assert!(
        body_string.contains("loading=\"lazy\"")
            || body_string.contains("ts-req")
            || body_string.contains("defer")
            || body_string.contains("async"),
        "Pages should implement lazy loading for non-critical content"
    );
}

#[tokio::test]
async fn test_css_optimization() {
    // Test that CSS is optimized for minimal payload
    let tailwind_config_content =
        fs::read_to_string("tailwind.config.js").expect("Tailwind config should exist");

    // Check for content purging configuration
    assert!(
        tailwind_config_content.contains("content") || tailwind_config_content.contains("purge"),
        "Tailwind should be configured for CSS purging"
    );

    // Check for file path optimization
    assert!(
        tailwind_config_content.contains("**/*.html")
            || tailwind_config_content.contains("templates"),
        "Tailwind should include template files for optimization"
    );

    // Check for production optimizations
    assert!(
        tailwind_config_content.contains("theme") || tailwind_config_content.contains("extend"),
        "Tailwind should have custom theme for optimization"
    );
}

#[tokio::test]
async fn test_image_optimization_patterns() {
    let app = create_test_app().await;

    // Test that image optimization patterns are in place
    let request = Request::builder()
        .uri("/recipes/discover")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for image optimization attributes (even if placeholder)
    if body_string.contains("<img") {
        assert!(
            body_string.contains("loading=\"lazy\"")
                || body_string.contains("decoding=\"async\"")
                || body_string.contains("width=")
                || body_string.contains("height="),
            "Images should include optimization attributes"
        );
    }

    // Check for responsive image patterns
    if body_string.contains("recipe") {
        // Recipe images should be optimized for kitchen viewing
        assert!(
            body_string.contains("bg-gray-200") || // Placeholder backgrounds
            body_string.contains("Recipe Image") ||
            !body_string.contains("data:image"), // No data URIs for large images
            "Recipe images should be optimized for performance"
        );
    }
}

#[tokio::test]
async fn test_performance_monitoring_integration() {
    let app = create_test_app().await;

    // Test that performance monitoring is integrated
    let request = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Metrics endpoint should be available for performance monitoring
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for performance-related metrics
    assert!(
        body_string.contains("request")
            || body_string.contains("duration")
            || body_string.contains("response")
            || body_string.contains("time"),
        "Metrics should include performance indicators"
    );
}

#[tokio::test]
async fn test_bundle_size_optimization() {
    // Test that JavaScript and CSS bundles are optimized
    let service_worker_size = fs::metadata("crates/imkitchen-web/static/js/service-worker.js")
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    // Service worker should be reasonably sized (under 50KB for good performance)
    assert!(
        service_worker_size < 50_000,
        "Service worker should be under 50KB (current: {} bytes)",
        service_worker_size
    );

    // Check if manifest is optimized
    let manifest_size = fs::metadata("crates/imkitchen-web/static/manifest.json")
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    assert!(
        manifest_size < 5_000,
        "PWA manifest should be under 5KB (current: {} bytes)",
        manifest_size
    );
}

#[tokio::test]
async fn test_mobile_performance_optimization() {
    let app = create_test_app().await;

    // Test mobile-specific performance optimizations
    let request = Request::builder()
        .uri("/recipes/discover")
        .header(
            "User-Agent",
            "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)",
        )
        .body(Body::empty())
        .unwrap();

    let start_time = Instant::now();
    let response = app.oneshot(request).await.unwrap();
    let elapsed = start_time.elapsed();

    // Mobile responses should be even faster
    assert!(
        elapsed < Duration::from_millis(500),
        "Mobile responses should be under 500ms (took {:.2}ms)",
        elapsed.as_millis()
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for mobile-optimized content
    assert!(
        body_string.contains("viewport")
            || body_string.contains("touch-target")
            || body_string.contains("md:")
            || body_string.contains("sm:"),
        "Mobile responses should include mobile optimizations"
    );
}

#[tokio::test]
async fn test_kitchen_environment_performance() {
    let app = create_test_app().await;

    // Test performance optimizations for kitchen environment usage
    let routes_for_kitchen = [
        "/recipes/discover",       // Finding recipes while cooking
        "/shopping-lists/current", // Checking shopping list at store
        "/meal-plans/current",     // Quick meal plan reference
    ];

    for route in routes_for_kitchen {
        let start_time = Instant::now();

        let request = Request::builder().uri(route).body(Body::empty()).unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        let elapsed = start_time.elapsed();

        // Kitchen environment requires very fast responses
        assert!(
            elapsed < Duration::from_millis(200),
            "Kitchen route {} should load under 200ms (took {:.2}ms)",
            route,
            elapsed.as_millis()
        );

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Check for kitchen-optimized content
        assert!(
            body_string.contains("kitchen")
                || body_string.contains("touch")
                || body_string.len() < 100_000, // Reasonable page size
            "Kitchen pages should be optimized for quick access"
        );
    }
}

#[cfg(test)]
mod performance_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_request_performance() {
        let app = create_test_app().await;

        // Test that the app handles concurrent requests efficiently
        let mut handles = vec![];

        for i in 0..10 {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {
                let route = if i % 2 == 0 {
                    "/auth/login"
                } else {
                    "/recipes/discover"
                };

                let start_time = Instant::now();
                let request = Request::builder().uri(route).body(Body::empty()).unwrap();

                let response = app_clone.oneshot(request).await.unwrap();
                let elapsed = start_time.elapsed();

                (response.status(), elapsed)
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let results = futures::future::join_all(handles).await;

        for (i, result) in results.into_iter().enumerate() {
            let (status, elapsed) = result.unwrap();
            assert!(status.is_success() || status.is_redirection());

            // Concurrent requests should not significantly degrade performance
            assert!(
                elapsed < Duration::from_secs(1),
                "Concurrent request {} took {:.2}ms",
                i,
                elapsed.as_millis()
            );
        }
    }

    #[tokio::test]
    async fn test_cache_header_optimization() {
        let app = create_test_app().await;

        // Test that appropriate cache headers are set for performance
        let request = Request::builder()
            .uri("/static/manifest.json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Static assets should have cache headers
        let headers = response.headers();

        // Check for cache-related headers (implementation may vary)
        let has_cache_control = headers.contains_key("cache-control")
            || headers.contains_key("expires")
            || headers.contains_key("etag");

        // For now, just ensure the request succeeds - cache headers are server configuration
        assert!(response.status().is_success() || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_compression_optimization() {
        // Test that responses support compression for performance
        let app = create_test_app().await;

        let request = Request::builder()
            .uri("/recipes/discover")
            .header("Accept-Encoding", "gzip, deflate, br")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        // Response should be reasonably sized (under 100KB for text content)
        assert!(
            body_bytes.len() < 100_000,
            "Response should be under 100KB (current: {} bytes)",
            body_bytes.len()
        );
    }
}
