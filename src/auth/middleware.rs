//! Authentication middleware for Axum

use super::jwt::{validate_token, AuthUser};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use tracing::{debug, error, warn};

/// Cookie name for JWT token
const AUTH_COOKIE_NAME: &str = "auth_token";

/// State for authentication middleware
#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
}

/// Authentication middleware that extracts JWT from cookie
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extract token from cookie
    let token = jar.get(AUTH_COOKIE_NAME).map(|cookie| cookie.value());

    let Some(token) = token else {
        warn!("No auth token found in request");
        return Err(Redirect::to("/auth/login").into_response());
    };

    // Validate token and extract user info
    match validate_token(token, &auth_state.jwt_secret) {
        Ok(auth_user) => {
            debug!(
                user_id = %auth_user.user_id,
                is_admin = auth_user.is_admin,
                "User authenticated"
            );

            // Insert user info into request extensions
            request.extensions_mut().insert(auth_user);

            Ok(next.run(request).await)
        }
        Err(e) => {
            error!(error = %e, "Invalid or expired token");
            Err(Redirect::to("/auth/login").into_response())
        }
    }
}

/// Extract authenticated user from request extensions
pub fn get_auth_user(request: &Request) -> Option<AuthUser> {
    request.extensions().get::<AuthUser>().cloned()
}
