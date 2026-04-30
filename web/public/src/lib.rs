pub use imkitchen_web_shared::config;

pub mod routes;

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::get;
    axum::Router::new()
        .route(
            "/upgrade",
            get(routes::upgrade::page).post(routes::upgrade::action),
        )
        .route(
            "/upgrade/order-summary",
            get(routes::upgrade::order_summary),
        )
        .route("/about", get(routes::about::page))
        .route("/help", get(routes::help::page))
        .route("/terms", get(routes::terms::page))
        .route("/policy", get(routes::policy::page))
        .route(
            "/contact",
            get(routes::contact::page).post(routes::contact::action),
        )
        .route(
            "/register",
            get(routes::register::page).post(routes::register::action),
        )
        .route(
            "/login",
            get(routes::login::page).post(routes::login::action),
        )
        .route(
            "/reset-password",
            get(routes::reset_password::page).post(routes::reset_password::action),
        )
        .route(
            "/reset-password/new/{id}",
            get(routes::reset_password::new_page).post(routes::reset_password::new_action),
        )
        .route("/logout", get(routes::login::logout))
        .route("/sw.js", get(routes::assets::service_worker))
        .route("/manifest.json", get(routes::assets::manifest))
        .route("/robots.txt", get(routes::assets::robots))
        .route("/sitemap.xml", get(routes::assets::sitemap))
}

pub fn health_routes() -> axum::Router<sqlx::SqlitePool> {
    use axum::routing::get;
    axum::Router::new()
        .route("/health", get(routes::health::health))
        .route("/_test-error", get(routes::health::test_error))
        .route("/ready", get(routes::health::ready))
}
