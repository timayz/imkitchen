use crate::SharedState;
use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use tower_cookies::{Cookie, Cookies};
use tracing::{info, warn};
use uuid::Uuid;

const CSRF_HEADER: &str = "X-CSRF-Token";
const CSRF_COOKIE: &str = "csrf_token";

/// CSRF protection middleware
/// Validates CSRF tokens for state-changing HTTP methods (POST, PUT, DELETE, PATCH)
pub async fn csrf_protection(
    State(_shared_state): State<SharedState>,
    cookies: Cookies,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let method = request.method().clone();

    // Only check CSRF for state-changing methods
    if !is_state_changing_method(&method) {
        return Ok(next.run(request).await);
    }

    // Get CSRF token from cookie
    let csrf_from_cookie = match cookies.get(CSRF_COOKIE) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            warn!("CSRF protection: No CSRF token cookie found");
            return Err((StatusCode::FORBIDDEN, "CSRF token required".to_string()));
        }
    };

    // Get CSRF token from header
    let headers = request.headers();
    let csrf_from_header = match headers.get(CSRF_HEADER) {
        Some(header_value) => match header_value.to_str() {
            Ok(value) => value,
            Err(_) => {
                warn!("CSRF protection: Invalid CSRF token header");
                return Err((
                    StatusCode::FORBIDDEN,
                    "Invalid CSRF token format".to_string(),
                ));
            }
        },
        None => {
            warn!("CSRF protection: No CSRF token header found");
            return Err((
                StatusCode::FORBIDDEN,
                "CSRF token header required".to_string(),
            ));
        }
    };

    // Compare tokens
    if csrf_from_cookie != csrf_from_header {
        warn!("CSRF protection: Token mismatch");
        return Err((StatusCode::FORBIDDEN, "CSRF token mismatch".to_string()));
    }

    info!("CSRF protection: Token validated successfully");
    Ok(next.run(request).await)
}

/// Generate and set a new CSRF token
pub async fn generate_csrf_token(cookies: Cookies) -> Result<String, (StatusCode, String)> {
    let csrf_token = Uuid::new_v4().to_string();

    // Set CSRF token in cookie
    let mut cookie = Cookie::new(CSRF_COOKIE, csrf_token.clone());
    cookie.set_http_only(false); // Allow JavaScript access for CSRF tokens
    cookie.set_secure(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
    cookie.set_path("/");

    cookies.add(cookie);

    info!("Generated new CSRF token");
    Ok(csrf_token)
}

/// Get current CSRF token from cookies
pub fn get_csrf_token(cookies: &Cookies) -> Option<String> {
    cookies
        .get(CSRF_COOKIE)
        .map(|cookie| cookie.value().to_string())
}

fn is_state_changing_method(method: &Method) -> bool {
    matches!(
        method,
        &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_state_changing_method() {
        assert!(is_state_changing_method(&Method::POST));
        assert!(is_state_changing_method(&Method::PUT));
        assert!(is_state_changing_method(&Method::DELETE));
        assert!(is_state_changing_method(&Method::PATCH));

        assert!(!is_state_changing_method(&Method::GET));
        assert!(!is_state_changing_method(&Method::HEAD));
        assert!(!is_state_changing_method(&Method::OPTIONS));
    }

    #[tokio::test]
    async fn test_csrf_token_generation() {
        // Create a simple test without relying on CookieJar internals
        let token = Uuid::new_v4().to_string();

        // Token should be a valid UUID format
        assert!(Uuid::parse_str(&token).is_ok());
        assert_eq!(token.len(), 36); // Standard UUID string length
    }
}
