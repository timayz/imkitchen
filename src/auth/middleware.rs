//! Authentication middleware for Axum

use super::jwt::{validate_token, AuthUser};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use imkitchen_user::aggregate::User;
use tracing::{debug, error, warn};

/// Cookie name for JWT token
const AUTH_COOKIE_NAME: &str = "auth_token";

/// State for authentication middleware
#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
    pub evento: evento::Sqlite,
}

/// Authentication middleware that extracts JWT from cookie and checks suspension
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
    let auth_user = match validate_token(token, &auth_state.jwt_secret) {
        Ok(user) => user,
        Err(e) => {
            error!(error = %e, "Invalid or expired token");
            return Err(Redirect::to("/auth/login").into_response());
        }
    };

    // Check if user is suspended by loading aggregate
    match evento::load::<User, _>(&auth_state.evento, &auth_user.user_id).await {
        Ok(user_result) => {
            if user_result.item.is_suspended {
                error!(
                    user_id = %auth_user.user_id,
                    "Suspended user attempted to access protected route"
                );
                // Redirect to login (cookie will be cleared client-side or on next auth)
                return Err(Redirect::to("/auth/login").into_response());
            }

            debug!(
                user_id = %auth_user.user_id,
                is_admin = auth_user.is_admin,
                "User authenticated and not suspended"
            );

            // Insert user info into request extensions
            request.extensions_mut().insert(auth_user);

            Ok(next.run(request).await)
        }
        Err(e) => {
            error!(
                error = %e,
                user_id = %auth_user.user_id,
                "Failed to load user aggregate"
            );
            Err(Redirect::to("/auth/login").into_response())
        }
    }
}

/// Extract authenticated user from request extensions
pub fn get_auth_user(request: &Request) -> Option<AuthUser> {
    request.extensions().get::<AuthUser>().cloned()
}
