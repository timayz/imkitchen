use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use rand::Rng;
use tracing::{info, warn};
use uuid::Uuid;

use crate::AppState;

/// Authentication middleware that checks for valid session
pub async fn auth_middleware(
    State(_app_state): State<AppState>,
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
                    if let Some(user_id) = extract_user_id_from_token(session_value) {
                        // Session token is valid and contains a valid user ID
                        info!("Valid session found for user: {}", user_id);
                        return next.run(request).await;
                    } else {
                        warn!("Invalid session token format: {}", session_value);
                    }
                }
            }
        }
    }

    // No valid session found, redirect to login
    info!(
        "No valid session found, redirecting to login for path: {}",
        path
    );
    Redirect::to("/auth/login").into_response()
}

/// Check if a route should be accessible without authentication
fn is_public_route(path: &str) -> bool {
    matches!(
        path,
        "/auth/login" | "/auth/register" | "/health" | "/metrics" | "/static"
    ) || path.starts_with("/static/")
}

/// Generate a cryptographically secure session token
pub fn generate_secure_session_token() -> String {
    let mut rng = rand::thread_rng();
    let token_bytes: [u8; 32] = rng.gen();
    BASE64_STANDARD.encode(token_bytes)
}

/// Create a session cookie header value with secure token
pub fn create_session_cookie_header(user_id: Uuid) -> String {
    let secure_token = generate_secure_session_token();
    // TODO: Store the mapping of secure_token -> user_id in database/cache
    // For now, we'll encode user_id in the token (this is temporary)
    let combined_token = format!("{}:{}", user_id, secure_token);
    let encoded_token = BASE64_STANDARD.encode(combined_token.as_bytes());

    format!(
        "imkitchen_session={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=604800; Secure",
        encoded_token
    )
}

/// Extract user ID from secure session token (temporary implementation)
pub fn extract_user_id_from_token(token: &str) -> Option<Uuid> {
    // Decode base64 token
    let decoded = BASE64_STANDARD.decode(token).ok()?;
    let combined = String::from_utf8(decoded).ok()?;

    // Split on ':' and extract UUID part
    let parts: Vec<&str> = combined.split(':').collect();
    if parts.len() == 2 {
        Uuid::parse_str(parts[0]).ok()
    } else {
        None
    }
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
    fn test_secure_session_token_generation() {
        let token1 = generate_secure_session_token();
        let token2 = generate_secure_session_token();

        // Tokens should be different each time
        assert_ne!(token1, token2);

        // Tokens should be base64 encoded and reasonable length
        assert!(token1.len() > 40);
        assert!(base64::Engine::decode(&BASE64_STANDARD, &token1).is_ok());
    }

    #[test]
    fn test_session_cookie_header_creation() {
        let user_id = Uuid::new_v4();
        let cookie_header = create_session_cookie_header(user_id);

        assert!(cookie_header.starts_with("imkitchen_session="));
        assert!(cookie_header.contains("HttpOnly"));
        assert!(cookie_header.contains("SameSite=Lax"));
        assert!(cookie_header.contains("Secure"));
    }

    #[test]
    fn test_token_extraction_roundtrip() {
        let user_id = Uuid::new_v4();
        let cookie_header = create_session_cookie_header(user_id);

        // Extract the token value from the cookie header
        let token_start =
            cookie_header.find("imkitchen_session=").unwrap() + "imkitchen_session=".len();
        let token_end = cookie_header.find(';').unwrap_or(cookie_header.len());
        let token = &cookie_header[token_start..token_end];

        // Should be able to extract the same user ID
        let extracted_user_id = extract_user_id_from_token(token);
        assert_eq!(extracted_user_id, Some(user_id));
    }
}
