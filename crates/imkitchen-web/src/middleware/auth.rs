use axum::{
    extract::{Request, State},
    http::{header, StatusCode, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use uuid::Uuid;
use tracing::{error, info, warn};

use crate::AppState;

/// Session key for storing user ID in cookies
const SESSION_COOKIE_NAME: &str = "imkitchen_session";

/// Authentication middleware that checks for valid session
pub async fn auth_middleware(
    State(app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    
    // Skip auth for public routes
    if is_public_route(path) {
        return next.run(request).await;
    }
    
    // Check for session cookie in headers
    let headers = request.headers();
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            // Parse cookies and find session cookie
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(session_value) = cookie.strip_prefix("imkitchen_session=") {
                    if let Ok(user_id) = Uuid::parse_str(session_value) {
                        // For now, assume session is valid if it parses as UUID
                        info!("Valid session found for user: {}", user_id);
                        return next.run(request).await;
                    } else {
                        warn!("Invalid session cookie format: {}", session_value);
                    }
                }
            }
        }
    }
    
    // No valid session found, redirect to login
    info!("No valid session found, redirecting to login for path: {}", path);
    Redirect::to("/auth/login").into_response()
}

/// Check if a route should be accessible without authentication
fn is_public_route(path: &str) -> bool {
    matches!(path, 
        "/auth/login" | 
        "/auth/register" | 
        "/health" | 
        "/metrics" |
        "/static" |
        _ if path.starts_with("/static/")
    )
}

/// Create a session cookie header value
pub fn create_session_cookie_header(user_id: Uuid) -> String {
    format!("imkitchen_session={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=604800", user_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_routes() {
        assert!(is_public_route("/auth/login"));
        assert!(is_public_route("/auth/register"));
        assert!(is_public_route("/health"));
        assert!(is_public_route("/metrics"));
        assert!(is_public_route("/static/css/main.css"));
        
        assert!(!is_public_route("/dashboard"));
        assert!(!is_public_route("/profile"));
        assert!(!is_public_route("/api/user"));
    }
    
    #[test]
    fn test_session_cookie_creation() {
        let user_id = Uuid::new_v4();
        let cookie = create_session_cookie(user_id);
        
        assert_eq!(cookie.name(), SESSION_COOKIE_NAME);
        assert_eq!(cookie.value(), user_id.to_string());
        assert!(cookie.http_only().unwrap());
        assert_eq!(cookie.path(), Some("/"));
    }
}