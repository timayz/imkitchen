pub mod routes;

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/admin/users", get(routes::users::page))
        .route("/admin/users/{id}/suspend", post(routes::users::suspend))
        .route("/admin/users/{id}/activate", post(routes::users::activate))
        .route("/admin/users/{id}/toggle-premium", post(routes::users::toggle_premium))
        .route("/admin/contact", get(routes::contact::page))
        .route("/admin/contact/{id}/mark-read-and-reply", post(routes::contact::mark_read_and_reply))
        .route("/admin/contact/{id}/resolve", post(routes::contact::resolve))
        .route("/admin/contact/{id}/reopen", post(routes::contact::reopen))
}
