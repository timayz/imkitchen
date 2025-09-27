use axum::{extract::State, http::StatusCode, response::Response};
use crate::metrics::AppMetrics;

/// Metrics endpoint handler that exports Prometheus metrics
pub async fn metrics_handler(
    State(metrics): State<AppMetrics>,
) -> Result<Response<String>, StatusCode> {
    let metrics_output = metrics.gather();
    
    if metrics_output.is_empty() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(Response::builder()
        .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
        .body(metrics_output)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let metrics = AppMetrics::new().unwrap();
        let result = metrics_handler(State(metrics)).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.body().contains("imkitchen"));
    }

    #[tokio::test]
    async fn test_metrics_content_type() {
        let metrics = AppMetrics::new().unwrap();
        let result = metrics_handler(State(metrics)).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "text/plain; version=0.0.4; charset=utf-8");
    }
}