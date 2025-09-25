use crate::SharedState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use imkitchen_core::services::AuthService;
use tower_cookies::{Cookie, Cookies};
use tracing::{info, warn};

/// Session validation middleware
/// Validates session cookies and ensures user is authenticated
pub async fn session_auth(
    State(shared_state): State<SharedState>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // Get session cookie
    let session_token = match cookies.get("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            warn!("No session cookie found");
            return Err((
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ));
        }
    };

    // Get database pool from shared state
    let app_state = shared_state.read().await;
    let db_pool = app_state
        .db
        .as_ref()
        .expect("Database must be initialized")
        .clone();
    drop(app_state);

    let auth_service = AuthService::new(db_pool);

    // Validate session
    match auth_service.validate_session(&session_token).await {
        Ok(user) => {
            info!("Session validated for user: {} ({})", user.name, user.email);

            // Add user info to request extensions for handlers to use
            request.extensions_mut().insert(user);

            // Continue to the protected route
            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!("Session validation failed: {}", e);

            // Clear invalid session cookie
            let mut cookie = Cookie::new("session_id", "");
            cookie.set_http_only(true);
            cookie.set_secure(true);
            cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
            cookie.set_path("/");
            // Set expiration to past time to delete cookie
            let past_time = tower_cookies::cookie::time::OffsetDateTime::from_unix_timestamp(
                (chrono::Utc::now() - chrono::Duration::days(1)).timestamp(),
            )
            .unwrap();
            cookie.set_expires(Some(past_time));
            cookies.add(cookie);

            Err((
                StatusCode::UNAUTHORIZED,
                "Session expired or invalid".to_string(),
            ))
        }
    }
}

/// Optional session middleware - doesn't fail if no session, but adds user info if valid
pub async fn optional_session_auth(
    State(shared_state): State<SharedState>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Response {
    // Get session cookie if present
    if let Some(session_cookie) = cookies.get("session_id") {
        let session_token = session_cookie.value();

        // Get database pool from shared state
        let app_state = shared_state.read().await;
        let db_pool = app_state
            .db
            .as_ref()
            .expect("Database must be initialized")
            .clone();
        drop(app_state);

        let auth_service = AuthService::new(db_pool);

        // Try to validate session
        match auth_service.validate_session(session_token).await {
            Ok(user) => {
                info!(
                    "Optional session validated for user: {} ({})",
                    user.name, user.email
                );
                request.extensions_mut().insert(user);
            }
            Err(e) => {
                warn!("Optional session validation failed: {}", e);
                // Don't fail the request, just continue without user info
            }
        }
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {}
