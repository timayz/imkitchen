pub use imkitchen_web_shared::config;

pub mod routes;

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/recipes", get(routes::index::page))
        .route("/recipes/create", post(routes::index::create))
        .route("/recipes/share-all", post(routes::index::share_all))
        .route(
            "/recipes/make-all-private",
            post(routes::index::make_all_private),
        )
        .route(
            "/recipes/import",
            get(routes::import::page).post(routes::import::action),
        )
        .route("/recipes/import/{id}/status", get(routes::import::status))
        .route(
            "/recipes/{id}/make-private",
            get(routes::detail::make_private_action),
        )
        .route(
            "/recipes/{id}/share-to-community",
            get(routes::detail::share_to_community_action),
        )
        .route(
            "/recipes/{id}/delete/status",
            get(routes::detail::delete_status),
        )
        .route(
            "/recipes/{id}/delete",
            get(routes::detail::delete_modal).post(routes::detail::delete_action),
        )
        .route("/recipes/{id}/save", post(routes::detail::save))
        .route("/recipes/{id}/unsave", post(routes::detail::unsave))
        .route(
            "/recipes/{id}/edit",
            get(routes::edit::page).post(routes::edit::action),
        )
        .route(
            "/recipes/{id}/thumbnail/{device}/image.webp",
            get(routes::thumbnail::get),
        )
        .route("/recipes/{id}/thumbnail", post(routes::thumbnail::upload))
        .route("/recipes/{id}", get(routes::detail::page))
        .route(
            "/recipes/_edit/ingredient-row",
            get(routes::edit::ingredient_row),
        )
        .route(
            "/recipes/_edit/instruction-row",
            get(routes::edit::instruction_row),
        )
}
