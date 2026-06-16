pub mod import;
pub mod routes;

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/admin/recipes/import", get(routes::recipe_import::page))
        .route(
            "/admin/recipes/import/{id}/status",
            get(routes::recipe_import::status),
        )
        .route("/admin/users", get(routes::users::page))
        .route("/admin/users/{id}/suspend", post(routes::users::suspend))
        .route("/admin/users/{id}/activate", post(routes::users::activate))
        .route(
            "/admin/users/{id}/toggle-premium",
            post(routes::users::toggle_premium),
        )
        .route("/admin/users/{id}/edit", get(routes::users::edit_modal))
        .route("/admin/users/{id}/role", post(routes::users::update_role))
        .route("/admin/invoices", get(routes::invoices::page))
        .route("/admin/invoices/{id}", get(routes::invoices::detail))
        .route("/admin/contact", get(routes::contact::page))
        .route(
            "/admin/contact/{id}/mark-read-and-reply",
            post(routes::contact::mark_read_and_reply),
        )
        .route(
            "/admin/contact/{id}/resolve",
            post(routes::contact::resolve),
        )
        .route("/admin/contact/{id}/reopen", post(routes::contact::reopen))
}

/// The ZIP upload endpoint is exported separately so the server can give it a larger request
/// body limit than the global cap (it is merged after the global limit layer).
pub fn upload_routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::post;
    axum::Router::new().route("/admin/recipes/import", post(routes::recipe_import::action))
}
