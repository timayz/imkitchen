use crate::middleware::csrf::{generate_csrf_token, get_csrf_token};
use axum::{http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfTokenResponse {
    pub token: String,
}

/// Get or generate a CSRF token
/// This endpoint provides CSRF tokens for authenticated users
pub async fn get_csrf_token_handler(
    cookies: Cookies,
) -> Result<Json<CsrfTokenResponse>, (StatusCode, String)> {
    info!("CSRF token requested");

    // Try to get existing token first
    let token = match get_csrf_token(&cookies) {
        Some(existing_token) => {
            info!("Returning existing CSRF token");
            existing_token
        }
        None => {
            info!("Generating new CSRF token");
            generate_csrf_token(cookies).await?
        }
    };

    Ok(Json(CsrfTokenResponse { token }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use serde_json::Value;
    use tower::ServiceExt;
    use tower_cookies::CookieManagerLayer;

    #[tokio::test]
    async fn test_get_csrf_token_handler() {
        let app = Router::new()
            .route("/csrf-token", get(get_csrf_token_handler))
            .layer(CookieManagerLayer::new());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/csrf-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let csrf_response: Value = serde_json::from_slice(&body).unwrap();

        assert!(csrf_response["token"].is_string());
        assert!(!csrf_response["token"].as_str().unwrap().is_empty());
    }
}
