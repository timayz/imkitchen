pub mod pages;
pub mod fragments;

use axum::Router;

pub fn create_routes() -> Router {
    Router::new()
        .merge(pages::create_page_routes())
        .merge(fragments::create_fragment_routes())
}