use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use user::validate_jwt;

use crate::routes::AppState;

/// Auth extension containing user_id extracted from JWT
#[derive(Clone, Debug)]
pub struct Auth {
    pub user_id: String,
}

/// Authentication middleware that validates JWT from cookie
///
/// Extracts auth_token cookie, validates JWT, verifies user exists in read model,
/// and inserts Auth extension with user_id
/// Redirects to /login if:
/// - Token is missing
/// - Token is invalid
/// - User does not exist in read model (deleted or not yet synced)
pub async fn auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Response {
    // Extract auth_token cookie
    let token = match jar.get("auth_token") {
        Some(cookie) => cookie.value(),
        None => {
            tracing::warn!("Missing auth_token cookie, redirecting to login");
            return (StatusCode::SEE_OTHER, [("Location", "/login")]).into_response();
        }
    };

    // Validate JWT and extract claims
    let claims = match validate_jwt(token, &state.jwt_secret) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Invalid JWT token: {:?}, redirecting to login", e);
            return (StatusCode::SEE_OTHER, [("Location", "/login")]).into_response();
        }
    };

    // Verify user exists in read model database
    // This catches:
    // 1. Deleted users (removed from read model)
    // 2. Users not yet synced from event store to read model
    let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?1")
        .bind(&claims.sub)
        .fetch_optional(&state.db_pool)
        .await;

    match user_exists {
        Ok(Some(_)) => {
            // User exists in read model - allow access
            req.extensions_mut().insert(Auth {
                user_id: claims.sub,
            });
            next.run(req).await
        }
        Ok(None) => {
            // User not found or deleted - redirect to login
            tracing::warn!(
                "User {} not found in read model (deleted or not synced), redirecting to login",
                claims.sub
            );
            (StatusCode::SEE_OTHER, [("Location", "/login")]).into_response()
        }
        Err(e) => {
            // Database error - redirect to login for safety
            tracing::error!(
                "Database error checking user existence: {:?}, redirecting to login",
                e
            );
            (StatusCode::SEE_OTHER, [("Location", "/login")]).into_response()
        }
    }
}
