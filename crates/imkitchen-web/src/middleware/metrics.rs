use crate::metrics::AppMetrics;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::debug;

pub async fn metrics_middleware(
    State(metrics): State<AppMetrics>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Track request in flight
    let _guard = metrics.start_http_request();

    // Process the request
    let response = next.run(request).await;

    // Record metrics
    let duration = start.elapsed();
    let status = response.status().as_u16();

    metrics.record_http_request(&method, &path, status, duration);

    debug!(
        method = %method,
        path = %path,
        status = %status,
        duration_ms = %duration.as_millis(),
        "HTTP request completed"
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::AppMetrics;
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode},
        middleware::from_fn_with_state,
        routing::get,
        Router,
    };
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_metrics_middleware() {
        let metrics = AppMetrics::new().unwrap();

        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(from_fn_with_state(metrics.clone(), metrics_middleware));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check that metrics were recorded
        let output = metrics.gather();
        assert!(output.contains("imkitchen_http_requests_total"));
        assert!(output.contains("imkitchen_http_request_duration_seconds"));
    }
}
