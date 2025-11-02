//! Admin route handlers

pub mod contact_inbox;
pub mod users;

use super::AppState;
use crate::auth::middleware::AuthState;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub fn admin_routes(auth_state: AuthState) -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(users::list_users))
        .route("/admin/users/{id}/suspend", post(users::suspend_user))
        .route("/admin/users/{id}/activate", post(users::activate_user))
        .route(
            "/admin/users/{id}/premium-bypass",
            post(users::toggle_premium_bypass),
        )
        .route("/admin/contact", get(contact_inbox::list_contact_messages))
        .route(
            "/admin/contact/{id}/mark-read",
            post(contact_inbox::mark_message_read),
        )
        .route(
            "/admin/contact/{id}/resolve",
            post(contact_inbox::resolve_message),
        )
        // Layers are applied in reverse order: last layer() runs first
        // So admin middleware runs first, then auth middleware
        .layer(middleware::from_fn(
            crate::middleware::admin::admin_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state,
            crate::auth::middleware::auth_middleware,
        ))
}
