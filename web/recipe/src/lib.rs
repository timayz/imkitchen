pub use imkitchen_web_shared::config;

pub mod routes;

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/recipes", get(routes::index::page))
        .route("/recipes/import", get(routes::import::page).post(routes::import::action))
        .route("/recipes/import/{id}/status", get(routes::import::status))
        .route("/recipes/{id}/make-private", get(routes::detail::make_private_action))
        .route("/recipes/{id}/share-to-community", get(routes::detail::share_to_community_action))
        .route("/recipes/{id}/delete/status", get(routes::detail::delete_status))
        .route("/recipes/{id}/delete", get(routes::detail::delete_modal).post(routes::detail::delete_action))
        .route("/recipes/{id}/check-in", post(routes::detail::check_in))
        .route("/recipes/{id}/check-like", post(routes::detail::check_like))
        .route("/recipes/{id}/uncheck-like", post(routes::detail::uncheck_like))
        .route("/recipes/{id}/check-unlike", post(routes::detail::check_unlike))
        .route("/recipes/{id}/uncheck-unlike", post(routes::detail::uncheck_unlike))
        .route("/recipes/{id}/save", post(routes::detail::save))
        .route("/recipes/{id}/unsave", post(routes::detail::unsave))
        .route("/recipes/{id}/edit", get(routes::edit::page).post(routes::edit::action))
        .route("/recipes/{id}/thumbnail/{device}/image.webp", get(routes::thumbnail::get))
        .route("/recipes/{id}/thumbnail", post(routes::thumbnail::upload))
        .route("/recipes/{id}/add-comment", get(routes::detail::add_comment_form).post(routes::detail::add_comment_action))
        .route("/recipes/{id}/add-comment-btn", get(routes::detail::add_comment_btn))
        .route("/recipes/{recipe_id}/reply/{comment_id}", get(routes::detail::reply_form).post(routes::detail::reply_action))
        .route("/recipes/{recipe_id}/cancel-reply/{comment_id}", get(routes::detail::cancel_reply))
        .route("/recipes/{id}/comments", get(routes::detail::comments))
        .route("/recipes/{recipe_id}/comments/{comment_id}/check-like", post(routes::detail::comment_check_like))
        .route("/recipes/{recipe_id}/comments/{comment_id}/uncheck-like", post(routes::detail::comment_uncheck_like))
        .route("/recipes/{recipe_id}/comments/{comment_id}/check-unlike", post(routes::detail::comment_check_unlike))
        .route("/recipes/{recipe_id}/comments/{comment_id}/uncheck-unlike", post(routes::detail::comment_uncheck_unlike))
        .route("/recipes/{id}", get(routes::detail::page))
        .route("/recipes/_edit/ingredient-row", get(routes::edit::ingredient_row))
        .route("/recipes/_edit/instruction-row", get(routes::edit::instruction_row))
}
