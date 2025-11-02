//! Authentication route handlers

pub mod login;
pub mod register;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use sqlx::SqlitePool;

// Re-export route handlers
pub use login::{get_login, post_login, post_logout};
pub use register::{get_register, get_register_status, post_register};

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub evento: evento::Sqlite,
    pub query_pool: SqlitePool,
    pub jwt_secret: String,
    pub jwt_lifetime_seconds: u64,
}

/// Helper to render templates
pub(crate) fn render_template<T: Template>(t: T) -> Response {
    match t.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {}", e),
        )
            .into_response(),
    }
}
