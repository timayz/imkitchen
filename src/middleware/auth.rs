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
/// Extracts auth_token cookie, validates JWT, and inserts Auth extension with user_id
/// Redirects to /login if token is missing or invalid
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
    match validate_jwt(token, &state.jwt_secret) {
        Ok(claims) => {
            // Insert Auth extension with user_id for downstream handlers
            req.extensions_mut().insert(Auth {
                user_id: claims.sub,
            });

            next.run(req).await
        }
        Err(e) => {
            tracing::warn!("Invalid JWT token: {:?}, redirecting to login", e);
            (StatusCode::SEE_OTHER, [("Location", "/login")]).into_response()
        }
    }
}
