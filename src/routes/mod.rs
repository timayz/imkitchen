use axum::{Router, response::IntoResponse, routing::post};
use sqlx::SqlitePool;

use crate::template::{NotFoundTemplate, Template};

mod admin;
mod health;
mod help;
mod index;
mod login;
mod policy;
mod profile;
mod register;
mod service_worker;
mod terms;

use axum::routing::get;

#[derive(Clone)]
pub struct AppState {
    pub config: crate::config::Config,
    pub user_command: imkitchen_user::Command<evento::Sqlite>,
    pub pool: SqlitePool,
}

pub async fn fallback(template: Template<NotFoundTemplate>) -> impl IntoResponse {
    template.render(NotFoundTemplate)
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        // Health check endpoints (no auth required)
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .with_state(app_state.pool.clone())
        .route("/", get(index::page))
        .route("/help", get(help::page))
        .route("/terms", get(terms::page))
        .route("/policy", get(policy::page))
        .route("/register", get(register::page).post(register::action))
        .route("/register/status/{id}", get(register::status))
        .route(
            "/login",
            get(login::page).post(crate::routes::login::action),
        )
        .route(
            "/profile/account",
            get(profile::account::page).post(profile::account::action),
        )
        .route(
            "/profile/meal-preferences",
            get(profile::meal_preferences::page).post(profile::meal_preferences::action),
        )
        .route(
            "/profile/subscription",
            get(profile::subscription::page).post(profile::subscription::action),
        )
        .route(
            "/profile/notifications",
            get(profile::notifications::page).post(profile::notifications::action),
        )
        .route(
            "/profile/security",
            get(profile::security::page).post(profile::security::action),
        )
        .route("/admin/users", get(admin::users::page))
        .route("/admin/users/suspend/{id}", post(admin::users::suspend))
        .route("/admin/users/activate/{id}", post(admin::users::activate))
        .route(
            "/admin/users/toggle-premium/{id}",
            post(admin::users::toggle_premium),
        )
        .fallback(fallback)
        .route("/sw.js", get(service_worker::sw))
        .nest_service("/static", crate::assets::AssetsService::new())
        .with_state(app_state)
}
