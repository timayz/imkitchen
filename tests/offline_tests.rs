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
async fn test_offline_page_exists() {
    let app = create_test_app().await;

    // Test that offline page is accessible
    let request = Request::builder()
        .uri("/offline")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for offline-specific content
    assert!(
        body_string.contains("offline") || body_string.contains("Offline"),
        "offline page should contain offline-related content"
    );

    // Check for offline features description
    assert!(
        body_string.contains("saved")
            || body_string.contains("cache")
            || body_string.contains("available"),
        "offline page should describe available offline features"
    );
}

#[tokio::test]
async fn test_offline_fallback_templates_exist() {
    // Test that offline fallback templates exist
    let offline_template_exists = fs::metadata("crates/imkitchen-web/templates/offline")
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false);

    assert!(
        offline_template_exists,
        "offline templates directory should exist"
    );

    // Check for specific offline templates
    let offline_index_exists = fs::metadata("crates/imkitchen-web/templates/offline/index.html")
        .map(|metadata| metadata.is_file())
        .unwrap_or(false);

    if !offline_index_exists {
        // Alternative check for a generic offline template
        let fallback_exists =
            fs::read_to_string("crates/imkitchen-web/templates/offline/fallback.html").is_ok()
                || fs::read_to_string("crates/imkitchen-web/templates/offline.html").is_ok();

        assert!(fallback_exists, "offline fallback template should exist");
    }
}

#[tokio::test]
async fn test_service_worker_offline_functionality() {
    // Test that service worker includes offline functionality
    let service_worker_content =
        fs::read_to_string("crates/imkitchen-web/static/js/service-worker.js")
            .expect("service worker should exist");

    // Check for offline-first caching strategy
    assert!(
        service_worker_content.contains("cache") && service_worker_content.contains("offline"),
        "service worker should implement offline caching"
    );

    // Check for cache strategies
    assert!(
        service_worker_content.contains("CacheFirst")
            || service_worker_content.contains("cache-first")
            || service_worker_content.contains("caches.match"),
        "service worker should use cache-first strategy"
    );

    // Check for fetch event handling
    assert!(
        service_worker_content.contains("fetch") && service_worker_content.contains("event"),
        "service worker should handle fetch events for offline support"
    );
}

#[tokio::test]
async fn test_offline_data_caching() {
    // Test that core meal planning data is cached for offline use
    let service_worker_content =
        fs::read_to_string("crates/imkitchen-web/static/js/service-worker.js")
            .expect("service worker should exist");

    // Check for meal planning data caching
    assert!(
        service_worker_content.contains("meal")
            || service_worker_content.contains("recipe")
            || service_worker_content.contains("cache_urls"),
        "service worker should cache meal planning data"
    );

    // Check for static asset caching
    assert!(
        service_worker_content.contains("static")
            || service_worker_content.contains("css")
            || service_worker_content.contains("js"),
        "service worker should cache static assets"
    );
}

#[tokio::test]
async fn test_connectivity_detection() {
    let app = create_test_app().await;

    // Test PWA offline status endpoint
    let request = Request::builder()
        .uri("/pwa/offline-status")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for connectivity status information
    assert!(
        body_string.contains("online")
            || body_string.contains("offline")
            || body_string.contains("connectivity"),
        "offline status endpoint should provide connectivity information"
    );
}

#[tokio::test]
async fn test_offline_mode_indicators() {
    // Test that templates include offline mode indicators
    let base_template_content =
        fs::read_to_string("crates/imkitchen-web/templates/layouts/base.html")
            .expect("base template should exist");

    // Check for offline detection scripts or indicators
    assert!(
        base_template_content.contains("offline")
            || base_template_content.contains("navigator.onLine")
            || base_template_content.contains("connectivity")
            || base_template_content.contains("network"),
        "base template should include offline mode detection"
    );
}

#[tokio::test]
async fn test_data_synchronization_patterns() {
    // Test that service worker includes data sync patterns
    let service_worker_content =
        fs::read_to_string("crates/imkitchen-web/static/js/service-worker.js")
            .expect("service worker should exist");

    // Check for background sync or sync patterns
    assert!(
        service_worker_content.contains("sync")
            || service_worker_content.contains("background")
            || service_worker_content.contains("queue")
            || service_worker_content.contains("retry"),
        "service worker should include data synchronization patterns"
    );

    // Check for sync event handling
    assert!(
        service_worker_content.contains("message")
            || service_worker_content.contains("postMessage")
            || service_worker_content.contains("event"),
        "service worker should handle sync events"
    );
}

#[tokio::test]
async fn test_offline_recipe_access() {
    let app = create_test_app().await;

    // Test that offline page indicates recipe access
    let request = Request::builder()
        .uri("/offline")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for offline recipe access information
    assert!(
        body_string.contains("recipe")
            || body_string.contains("meal")
            || body_string.contains("saved"),
        "offline page should indicate recipe access capabilities"
    );
}

#[tokio::test]
async fn test_offline_meal_plan_access() {
    let app = create_test_app().await;

    // Test that offline functionality supports meal planning
    let request = Request::builder()
        .uri("/offline")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for offline meal planning capabilities
    assert!(
        body_string.contains("meal")
            || body_string.contains("plan")
            || body_string.contains("week"),
        "offline page should support meal planning access"
    );
}

#[tokio::test]
async fn test_offline_cache_management() {
    let app = create_test_app().await;

    // Test cache status endpoint
    let request = Request::builder()
        .uri("/pwa/cache-status")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check for cache management information
    assert!(
        body_string.contains("cache")
            || body_string.contains("storage")
            || body_string.contains("size"),
        "cache status should provide cache management information"
    );
}

#[cfg(test)]
mod offline_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_offline_first_strategy() {
        // Test that the app implements offline-first strategy
        let service_worker_content =
            fs::read_to_string("crates/imkitchen-web/static/js/service-worker.js")
                .expect("service worker should exist");

        // Check for offline-first patterns
        assert!(
            service_worker_content.contains("caches.open")
                || service_worker_content.contains("cache.add")
                || service_worker_content.contains("addAll"),
            "service worker should implement offline-first caching"
        );

        // Check for cache versioning
        assert!(
            service_worker_content.contains("version")
                || service_worker_content.contains("v1")
                || service_worker_content.contains("CACHE_NAME"),
            "service worker should implement cache versioning"
        );
    }

    #[tokio::test]
    async fn test_offline_error_handling() {
        // Test that offline errors are handled gracefully
        let app = create_test_app().await;

        // Test offline page error handling
        let request = Request::builder()
            .uri("/offline")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Check for offline error handling guidance
        assert!(
            body_string.contains("error")
                || body_string.contains("problem")
                || body_string.contains("try")
                || body_string.contains("connection"),
            "offline page should provide error handling guidance"
        );
    }

    #[tokio::test]
    async fn test_offline_navigation_fallback() {
        // Test that navigation works offline through cached content
        let service_worker_content =
            fs::read_to_string("crates/imkitchen-web/static/js/service-worker.js")
                .expect("service worker should exist");

        // Check for navigation fallback patterns
        assert!(
            service_worker_content.contains("navigate")
                || service_worker_content.contains("fallback")
                || service_worker_content.contains("offline.html")
                || service_worker_content.contains("request.mode"),
            "service worker should provide navigation fallbacks"
        );
    }

    #[tokio::test]
    async fn test_offline_storage_optimization() {
        // Test that offline storage is optimized for kitchen use
        let service_worker_content =
            fs::read_to_string("crates/imkitchen-web/static/js/service-worker.js")
                .expect("service worker should exist");

        // Check for storage optimization patterns
        assert!(
            service_worker_content.contains("cleanup")
                || service_worker_content.contains("expire")
                || service_worker_content.contains("maxEntries")
                || service_worker_content.contains("delete"),
            "service worker should optimize offline storage"
        );
    }
}
