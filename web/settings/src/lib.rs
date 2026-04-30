pub use imkitchen_web_shared::config;

pub mod routes;

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route(
            "/settings/general/set-username",
            post(routes::general::set_username_action),
        )
        .route(
            "/settings/general",
            get(routes::general::page).post(routes::general::action),
        )
        .route("/settings/billing", get(routes::billing::page))
        .route("/settings/billing/check", post(routes::billing::check))
        .route(
            "/settings/billing/payment-method",
            post(routes::billing::payment_method),
        )
        .route(
            "/settings/billing/cancel",
            get(routes::billing::cancel_modal).post(routes::billing::cancel),
        )
        .route(
            "/settings/billing/update-payment",
            get(routes::billing::update_payment_modal).post(routes::billing::update_payment),
        )
        .route(
            "/settings/account",
            get(routes::account::page).post(routes::account::action),
        )
        .route("/settings/recipes", get(routes::recipes::page))
        .route("/settings/recipes/create", post(routes::recipes::create))
        .route("/invoices/{id}", get(routes::invoices::detail::page))
}
