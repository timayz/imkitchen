pub mod health;
pub mod home;

use axum::{routing::get, Router};

pub fn create_page_routes() -> Router {
    Router::new()
        .route("/", get(home::home_page))
        .route("/health", get(health::health_check))
}
