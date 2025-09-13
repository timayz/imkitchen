pub mod auth;
pub mod health;
pub mod home;

use axum::{routing::get, Router};

pub fn create_page_routes() -> Router {
    Router::new()
        .route("/", get(home::home_page))
        .route("/health", get(health::health_check))
}

pub fn create_auth_routes() -> Router<crate::services::AuthService> {
    auth::auth_routes()
}
