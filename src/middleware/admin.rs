//! Admin authorization middleware for Axum

use crate::auth::jwt::AuthUser;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::{error, warn};

/// Admin middleware that verifies user has admin privileges
pub async fn admin_middleware(request: Request, next: Next) -> Result<Response, Response> {
    // Extract AuthUser from request extensions (set by auth_middleware)
    let auth_user = request
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or_else(|| {
            warn!("Admin middleware: No authenticated user found in request extensions");
            (
                StatusCode::UNAUTHORIZED,
                "Authentication required to access admin panel",
            )
                .into_response()
        })?;

    // Check if user is admin
    if !auth_user.is_admin {
        error!(
            user_id = %auth_user.user_id,
            "Non-admin user attempted to access admin route"
        );
        return Err((
            StatusCode::FORBIDDEN,
            "Admin privileges required to access this resource",
        )
            .into_response());
    }

    // User is admin - allow request to proceed
    Ok(next.run(request).await)
}
