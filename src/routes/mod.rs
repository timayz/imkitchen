use axum::{
    Router,
    response::IntoResponse,
    routing::{get, post},
};
use sqlx::SqlitePool;

use crate::template::{NotFoundTemplate, Template};

mod about;
mod admin;
mod calendar;
mod contact;
mod health;
mod help;
mod index;
mod kitchen;
mod login;
mod manifest;
mod policy;
mod profile;
mod recipes;
mod register;
mod reset_password;
mod service_worker;
mod shopping;
mod terms;

#[derive(Clone)]
pub struct AppState {
    pub config: crate::config::Config,
    pub user_command: imkitchen_user::Command<evento::sql::RwSqlite>,
    pub user_subscription_command: imkitchen_user::subscription::Command<evento::sql::RwSqlite>,
    pub user_meal_preference_command:
        imkitchen_user::meal_preferences::Command<evento::sql::RwSqlite>,
    pub user_reset_password_command: imkitchen_user::reset_password::Command<evento::sql::RwSqlite>,
    pub user_query: imkitchen_user::Query,
    pub contact_command: imkitchen_contact::Command<evento::sql::RwSqlite>,
    pub contact_query: imkitchen_contact::Query,
    pub recipe_command: imkitchen_recipe::Command<evento::sql::RwSqlite>,
    pub recipe_query: imkitchen_recipe::Query,
    pub mealplan_command: imkitchen_mealplan::Command<evento::sql::RwSqlite>,
    pub mealplan_query: imkitchen_mealplan::Query,
    pub shopping_command: imkitchen_shopping::Command<evento::sql::RwSqlite>,
    pub shopping_query: imkitchen_shopping::Query,
    pub pool: SqlitePool,
}

pub async fn fallback(template: Template) -> impl IntoResponse {
    template.render(NotFoundTemplate)
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        // Health check endpoints (no auth required)
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .with_state(app_state.pool.clone())
        .route("/", get(index::page))
        .route("/kitchen/{day}", get(kitchen::page))
        .route("/about", get(about::page))
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
        .route(
            "/reset-password",
            get(reset_password::page).post(reset_password::action),
        )
        .route(
            "/reset-password/new/{id}",
            get(reset_password::new_page).post(reset_password::new_action),
        )
        .route(
            "/calendar/regenerate",
            get(calendar::regenerate_modal).post(calendar::regenerate_action),
        )
        .route(
            "/calendar/regenerate/status",
            get(calendar::regenerate_status),
        )
        .route("/calendar/week-{index}", get(calendar::page))
        .route(
            "/calendar/week-{index}/shopping",
            get(shopping::page).post(shopping::reset_all_action),
        )
        .route(
            "/calendar/week-{timestamp}/shopping/toggle",
            post(shopping::toggle_action),
        )
        .route("/recipes", get(recipes::index::page))
        .route("/recipes/community", get(recipes::community::page))
        .route("/recipes/create", post(recipes::index::create))
        .route(
            "/recipes/create/{id}/status",
            get(recipes::index::create_status),
        )
        .route(
            "/recipes/create-mobile",
            post(recipes::index::create_mobile),
        )
        .route(
            "/recipes/create-mobile/{id}/status",
            get(recipes::index::create_mobile_status),
        )
        .route(
            "/recipes/import",
            get(recipes::import::page).post(recipes::import::action),
        )
        .route("/recipes/import/{id}/status", get(recipes::import::status))
        .route("/recipes/{id}/detail", get(recipes::detail::page))
        .route(
            "/recipes/{id}/detail/make-private",
            get(recipes::detail::make_private_action),
        )
        .route(
            "/recipes/{id}/detail/share-to-community",
            get(recipes::detail::share_to_community_action),
        )
        .route(
            "/recipes/{id}/detail/delete/status",
            get(recipes::detail::delete_status),
        )
        .route(
            "/recipes/{id}/detail/delete",
            get(recipes::detail::delete_modal).post(recipes::detail::delete_action),
        )
        .route(
            "/recipes/{id}/community",
            get(recipes::community_detail::page),
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
            "/profile/account/set-username",
            post(profile::account::set_username_action),
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
        .route("/sw.js", get(service_worker::asset))
        .route("/manifest.json", get(manifest::asset))
        .nest_service("/static", crate::assets::AssetsService::new())
        .with_state(app_state)
}
