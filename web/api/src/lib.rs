//! JSON API consumed by the native mobile app (`mobile/`).
//!
//! Every route lives under `/api/v1`, speaks JSON only and authenticates with a
//! Bearer token: the Lynx `fetch` has no cookie jar and does not follow
//! redirects, so nothing here may answer with a `Redirect` or rely on cookies.
//! Handlers are thin adapters over the framework-agnostic services in
//! `imkitchen_web_shared`, shared with the HTML routes.

pub mod auth;
pub mod dto;
pub mod error;
pub mod json;
pub mod routes;

pub use error::{ApiError, ApiResult};
pub use json::ApiJson;

use axum::http::{HeaderValue, header};
use axum::response::Response;

/// API responses are personal and never cacheable; the shared
/// `cache_control_middleware` yields to a handler-set header.
async fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// The thumbnail upload is exported separately so the server can give it a
/// larger request body limit than the global 1 MB cap (it is merged after
/// the global limit layer, like the admin ZIP upload).
pub fn upload_routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::post;

    axum::Router::new()
        .route(
            "/api/v1/recipes/{id}/thumbnail",
            post(routes::recipe::thumbnail),
        )
        .layer(axum::middleware::map_response(no_store))
}

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};

    let v1 = axum::Router::new()
        .route("/health", get(routes::health::health))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/refresh", post(routes::auth::refresh))
        .route("/auth/password-reset", post(routes::auth::password_reset))
        .route(
            "/auth/password-reset/{id}",
            get(routes::auth::password_reset_check).post(routes::auth::password_reset_confirm),
        )
        .route("/me", get(routes::auth::me))
        .route("/kitchen", get(routes::kitchen::overview))
        .route("/kitchen/generate", post(routes::kitchen::generate))
        .route(
            "/kitchen/recipes/{id}",
            get(routes::kitchen::dish).delete(routes::kitchen::remove),
        )
        .route("/kitchen/recipes/{id}/cook", get(routes::kitchen::cook))
        .route("/kitchen/recipes/{id}/step", post(routes::kitchen::step))
        .route(
            "/kitchen/recipes/{id}/status",
            axum::routing::put(routes::kitchen::set_status),
        )
        .route("/groceries", get(routes::grocery::list))
        .route("/groceries/toggle", post(routes::grocery::toggle))
        .route(
            "/groceries/check",
            axum::routing::put(routes::grocery::check),
        )
        .route(
            "/recipes",
            get(routes::recipe::browse).post(routes::recipe::create),
        )
        .route("/recipes/share-all", post(routes::recipe::share_all))
        .route("/recipes/unshare-all", post(routes::recipe::unshare_all))
        .route("/recipes/import", post(routes::recipe::import))
        .route(
            "/recipes/{id}",
            get(routes::recipe::detail)
                .put(routes::recipe::update)
                .delete(routes::recipe::delete),
        )
        .route("/recipes/{id}/edit", get(routes::recipe::edit))
        .route("/recipes/{id}/similar", get(routes::recipe::similar))
        .route("/recipes/{id}/exists", get(routes::recipe::exists))
        .route(
            "/recipes/{id}/save",
            post(routes::recipe::save).delete(routes::recipe::unsave),
        )
        .route(
            "/recipes/{id}/shopping",
            post(routes::recipe::add_to_shopping),
        )
        .route("/recipes/{id}/share", post(routes::recipe::share))
        .route("/recipes/{id}/unshare", post(routes::recipe::unshare))
        .route("/cooks/{username}", get(routes::recipe::cook))
        .route("/settings/general", get(routes::settings::general))
        .route(
            "/settings/preferences",
            axum::routing::put(routes::settings::preferences),
        )
        .route(
            "/settings/profile",
            axum::routing::put(routes::settings::profile),
        )
        .route(
            "/settings/aisles",
            axum::routing::put(routes::settings::aisle_order),
        )
        .route("/settings/username", post(routes::settings::username))
        .route(
            "/settings/account",
            axum::routing::delete(routes::settings::delete_account),
        )
        .route(
            "/settings/account/password-reset",
            post(routes::settings::password_reset),
        )
        .route("/settings/sessions", get(routes::settings::sessions))
        .route(
            "/settings/sessions/{acc}",
            axum::routing::delete(routes::settings::revoke_session),
        )
        .fallback(routes::health::not_found)
        .layer(axum::middleware::map_response(no_store));

    axum::Router::new().nest("/api/v1", v1)
}
