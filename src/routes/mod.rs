use axum::{Router, response::IntoResponse, routing::post};
use sqlx::SqlitePool;

use crate::template::{NotFoundTemplate, Template};

mod admin;
mod calendar;
mod community;
mod contact;
mod health;
mod help;
mod index;
mod login;
mod policy;
mod profile;
mod recipes;
mod register;
mod reset_password;
mod service_worker;
mod terms;

use axum::routing::get;

#[derive(Clone)]
pub struct AppState {
    pub config: crate::config::Config,
    pub user_command: imkitchen_user::Command<evento::Sqlite>,
    pub user_subscription_command: imkitchen_user::subscription::Command<evento::Sqlite>,
    pub user_meal_preference_command: imkitchen_user::meal_preferences::Command<evento::Sqlite>,
    pub user_query: imkitchen_user::Query,
    pub contact_command: imkitchen_contact::Command<evento::Sqlite>,
    pub contact_query: imkitchen_contact::Query,
    pub recipe_command: imkitchen_recipe::Command<evento::Sqlite>,
    pub recipe_query: imkitchen_recipe::Query,
    // pub mealplan_command: imkitchen_mealplan::Command<evento::Sqlite>,
    // pub mealplan_query: imkitchen_mealplan::Query,
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
        .route("/contact", get(contact::page).post(contact::action))
        .route("/register", get(register::page).post(register::action))
        .route("/register/{id}/status", get(register::status))
        .route(
            "/login",
            get(login::page).post(crate::routes::login::action),
        )
        .route("/reset-password", get(reset_password::page))
        .route("/calendar", get(calendar::page))
        .route("/community", get(community::page))
        .route("/recipes", get(recipes::index::page))
        .route("/recipes/create", get(recipes::index::create))
        .route(
            "/recipes/create/{id}/status",
            get(recipes::index::create_status),
        )
        .route(
            "/recipes/import",
            get(recipes::import::page).post(recipes::import::action),
        )
        .route("/recipes/import/{id}/status", get(recipes::import::status))
        .route("/recipes/{id}/detail", get(recipes::detail::page))
        .route(
            "/recipes/{id}/delete/status",
            get(recipes::detail::delete_status),
        )
        .route(
            "/recipes/{id}/delete",
            get(recipes::detail::delete_modal).post(recipes::detail::delete_action),
        )
        .route(
            "/recipes/{id}/edit",
            get(recipes::edit::page).post(recipes::edit::action),
        )
        .route(
            "/recipes/_edit/ingredient-row",
            get(recipes::edit::ingredient_row),
        )
        .route(
            "/recipes/_edit/instruction-row",
            get(recipes::edit::instruction_row),
        )
        .route("/logout", get(login::logout))
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
        .route("/admin/users/{id}/suspend", post(admin::users::suspend))
        .route("/admin/users/{id}/activate", post(admin::users::activate))
        .route(
            "/admin/users/{id}/toggle-premium",
            post(admin::users::toggle_premium),
        )
        .route("/admin/contact", get(admin::contact::page))
        .route(
            "/admin/contact/{id}/mark-read-and-reply",
            post(admin::contact::mark_read_and_reply),
        )
        .route("/admin/contact/{id}/resolve", post(admin::contact::resolve))
        .route("/admin/contact/{id}/reopen", post(admin::contact::reopen))
        .fallback(fallback)
        .route("/sw.js", get(service_worker::sw))
        .nest_service("/static", crate::assets::AssetsService::new())
        .with_state(app_state)
}
