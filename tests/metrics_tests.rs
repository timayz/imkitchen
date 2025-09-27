use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use imkitchen_web::{create_app_with_metrics, AppMetrics};
use std::time::Duration;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_metrics_endpoint() {
    let metrics = AppMetrics::new().unwrap();

    // Add some test data to ensure metrics are present
    metrics.record_http_request("GET", "/test", 200, Duration::from_millis(50));
    metrics.update_db_connections(5, 2);
    metrics.set_app_info("1.0.0", "1.70.0");

    let app = create_app_with_metrics(None, metrics);

    let request = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Check for expected prometheus metrics
    assert!(body_str.contains("imkitchen_http_requests_total"));
    assert!(body_str.contains("imkitchen_http_request_duration_seconds"));
    assert!(body_str.contains("imkitchen_db_connections_active"));
    assert!(body_str.contains("imkitchen_app_info"));
}

#[tokio::test]
async fn test_metrics_middleware_integration() {
    let metrics = AppMetrics::new().unwrap();
    let app = create_app_with_metrics(None, metrics.clone());

    // Make a request to the health endpoint
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check that metrics were recorded
    let metrics_output = metrics.gather();
    assert!(metrics_output.contains("imkitchen_http_requests_total"));
    assert!(metrics_output.contains("/health"));
}

#[tokio::test]
async fn test_metrics_http_request_tracking() {
    let metrics = AppMetrics::new().unwrap();

    // Record some test metrics
    metrics.record_http_request("GET", "/test", 200, Duration::from_millis(50));
    metrics.record_http_request("POST", "/api/test", 201, Duration::from_millis(100));
    metrics.record_http_request("GET", "/error", 500, Duration::from_millis(25));

    let output = metrics.gather();

    // Check that metrics contain our test data
    assert!(output.contains("imkitchen_http_requests_total"));
    assert!(output.contains("imkitchen_http_request_duration_seconds"));
    assert!(output.contains("GET"));
    assert!(output.contains("POST"));
    assert!(output.contains("200"));
    assert!(output.contains("201"));
    assert!(output.contains("500"));
}

#[tokio::test]
async fn test_metrics_database_tracking() {
    let metrics = AppMetrics::new().unwrap();

    // Test database metrics
    metrics.update_db_connections(5, 2);
    metrics.record_db_query("select", "success", Duration::from_millis(10));
    metrics.record_db_query("insert", "error", Duration::from_millis(50));

    let output = metrics.gather();

    // Check database metrics
    assert!(output.contains("imkitchen_db_connections_active"));
    assert!(output.contains("imkitchen_db_connections_idle"));
    assert!(output.contains("imkitchen_db_queries_total"));
    assert!(output.contains("imkitchen_db_query_duration_seconds"));
    assert!(output.contains("select"));
    assert!(output.contains("insert"));
    assert!(output.contains("success"));
    assert!(output.contains("error"));
}

#[tokio::test]
async fn test_metrics_health_check_tracking() {
    let metrics = AppMetrics::new().unwrap();

    // Test health check metrics
    metrics.record_health_check("database", 2, Duration::from_millis(5)); // healthy
    metrics.record_health_check("system", 1, Duration::from_millis(15)); // degraded

    let output = metrics.gather();

    // Check health check metrics
    assert!(output.contains("imkitchen_health_check_status"));
    assert!(output.contains("imkitchen_health_check_duration_seconds"));
    assert!(output.contains("database"));
    assert!(output.contains("system"));
}

#[tokio::test]
async fn test_metrics_event_processing() {
    let metrics = AppMetrics::new().unwrap();

    // Test event processing metrics
    metrics.record_event_processed("UserCreated", "success", Duration::from_millis(3));
    metrics.record_event_processed("OrderPlaced", "error", Duration::from_millis(8));

    let output = metrics.gather();

    // Check event processing metrics
    assert!(output.contains("imkitchen_events_processed_total"));
    assert!(output.contains("imkitchen_event_processing_duration_seconds"));
    assert!(output.contains("UserCreated"));
    assert!(output.contains("OrderPlaced"));
}

#[tokio::test]
async fn test_metrics_guards() {
    let metrics = AppMetrics::new().unwrap();

    // Test HTTP request guard
    {
        let guard = metrics.start_http_request();
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(1)).await;
        guard.complete("GET", "/test-guard", 200);
    }

    // Test DB query guard
    {
        let guard = metrics.start_db_query("test_query");
        tokio::time::sleep(Duration::from_millis(1)).await;
        guard.complete("success");
    }

    // Test event processing guard
    {
        let guard = metrics.start_event_processing("TestEvent");
        tokio::time::sleep(Duration::from_millis(1)).await;
        guard.complete("success");
    }

    let output = metrics.gather();

    // Verify all guard metrics were recorded
    assert!(output.contains("imkitchen_http_requests_total"));
    assert!(output.contains("/test-guard"));
    assert!(output.contains("test_query"));
    assert!(output.contains("TestEvent"));
}

#[tokio::test]
async fn test_metrics_app_info() {
    let metrics = AppMetrics::new().unwrap();

    // Set app info
    metrics.set_app_info("1.0.0", "1.70.0");
    metrics.update_uptime(Duration::from_secs(3600)); // 1 hour

    let output = metrics.gather();

    // Check app info metrics
    assert!(output.contains("imkitchen_app_info"));
    assert!(output.contains("imkitchen_uptime_seconds"));
    assert!(output.contains("1.0.0"));
    assert!(output.contains("1.70.0"));
}
