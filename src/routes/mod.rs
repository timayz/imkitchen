use std::ops::Deref;

use axum::{
    Router,
    response::IntoResponse,
    routing::{get, post},
};
use evento::sql::RwSqlite;

use crate::template::{NotFoundTemplate, Template};

mod about;
mod admin;
mod assets;
mod contact;
mod groceries;
mod health;
mod help;
mod index;
mod invoices;
mod login;
mod menu;
mod policy;
mod recipes;
mod register;
mod reset_password;
mod settings;
mod terms;
mod upgrade;

#[derive(Clone)]
pub struct AppState {
    pub inner: imkitchen_shared::State<RwSqlite>,
    pub config: crate::config::Config,
    pub stripe: stripe::Client,
    pub identity: imkitchen_identity::Module<RwSqlite>,
    pub billing: imkitchen_billing::Billing<RwSqlite>,
    pub core: imkitchen_core::Core<RwSqlite>,
}

impl Deref for AppState {
    type Target = imkitchen_shared::State<RwSqlite>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub async fn fallback(template: Template) -> impl IntoResponse {
    template.render(NotFoundTemplate)
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        // Health check endpoints (no auth required)
        .route("/health", get(health::health))
        .route("/_test-error", get(health::test_error))
        .route("/ready", get(health::ready))
        .with_state(app_state.read_db.clone())
        .route("/", get(index::page))
        .route(
            "/kitchen/{date}/{recipe_id}/step/{direction}",
            post(index::update_slot_step_action),
        )
        .route(
            "/kitchen/{date}/{recipe_id}/select-dish",
            get(index::select_dish),
        )
        .route("/kitchen/{date}", get(index::page))
        .route("/upgrade", get(upgrade::page).post(upgrade::action))
        .route("/upgrade/order-summary", get(upgrade::order_summary))
        .route("/about", get(about::page))
        .route("/help", get(help::page))
        .route("/terms", get(terms::page))
        .route("/policy", get(policy::page))
        .route("/contact", get(contact::page).post(contact::action))
        .route("/register", get(register::page).post(register::action))
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
        .route("/menu", get(menu::page))
        .route("/menu/{date}", get(menu::page))
        .route(
            "/menu/{date}/generate",
            get(menu::generate_modal).post(menu::generate_action),
        )
        .route("/menu/{date}/generate/status", get(menu::generate_status))
        .route("/groceries", get(groceries::page))
        .route("/groceries/toggle", post(groceries::toggle_action))
        .route(
            "/groceries/generate",
            get(groceries::generate_modal).post(groceries::generate_action),
        )
        .route(
            "/groceries/generate/status",
            get(groceries::generate_status),
        )
        .route("/recipes", get(recipes::index::page))
        .route(
            "/recipes/import",
            get(recipes::import::page).post(recipes::import::action),
        )
        .route("/recipes/import/{id}/status", get(recipes::import::status))
        .route(
            "/recipes/{id}/make-private",
            get(recipes::detail::make_private_action),
        )
        .route(
            "/recipes/{id}/share-to-community",
            get(recipes::detail::share_to_community_action),
        )
        .route(
            "/recipes/{id}/delete/status",
            get(recipes::detail::delete_status),
        )
        .route(
            "/recipes/{id}/delete",
            get(recipes::detail::delete_modal).post(recipes::detail::delete_action),
        )
        .route("/recipes/{id}/check-in", post(recipes::detail::check_in))
        .route(
            "/recipes/{id}/check-like",
            post(recipes::detail::check_like),
        )
        .route(
            "/recipes/{id}/uncheck-like",
            post(recipes::detail::uncheck_like),
        )
        .route(
            "/recipes/{id}/check-unlike",
            post(recipes::detail::check_unlike),
        )
        .route(
            "/recipes/{id}/uncheck-unlike",
            post(recipes::detail::uncheck_unlike),
        )
        .route("/recipes/{id}/save", post(recipes::detail::save))
        .route("/recipes/{id}/unsave", post(recipes::detail::unsave))
        .route(
            "/recipes/{id}/edit",
            get(recipes::edit::page).post(recipes::edit::action),
        )
        .route(
            "/recipes/{id}/thumbnail/{device}/image.webp",
            get(recipes::thumbnail::get),
        )
        .route("/recipes/{id}/thumbnail", post(recipes::thumbnail::upload))
        .route(
            "/recipes/{id}/add-comment",
            get(recipes::detail::add_comment_form).post(recipes::detail::add_comment_action),
        )
        .route(
            "/recipes/{id}/add-comment-btn",
            get(recipes::detail::add_comment_btn),
        )
        .route(
            "/recipes/{recipe_id}/reply/{comment_id}",
            get(recipes::detail::reply_form).post(recipes::detail::reply_action),
        )
        .route(
            "/recipes/{recipe_id}/cancel-reply/{comment_id}",
            get(recipes::detail::cancel_reply),
        )
        .route("/recipes/{id}/comments", get(recipes::detail::comments))
        .route(
            "/recipes/{recipe_id}/comments/{comment_id}/check-like",
            post(recipes::detail::comment_check_like),
        )
        .route(
            "/recipes/{recipe_id}/comments/{comment_id}/uncheck-like",
            post(recipes::detail::comment_uncheck_like),
        )
        .route(
            "/recipes/{recipe_id}/comments/{comment_id}/check-unlike",
            post(recipes::detail::comment_check_unlike),
        )
        .route(
            "/recipes/{recipe_id}/comments/{comment_id}/uncheck-unlike",
            post(recipes::detail::comment_uncheck_unlike),
        )
        .route("/recipes/{id}", get(recipes::detail::page))
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
            "/settings/general/set-username",
            post(settings::general::set_username_action),
        )
        .route(
            "/settings/general",
            get(settings::general::page).post(settings::general::action),
        )
        .route("/settings/billing", get(settings::billing::page))
        .route("/settings/billing/check", post(settings::billing::check))
        .route(
            "/settings/billing/payment-method",
            post(settings::billing::payment_method),
        )
        .route(
            "/settings/billing/cancel",
            get(settings::billing::cancel_modal).post(settings::billing::cancel),
        )
        .route(
            "/settings/billing/update-payment",
            get(settings::billing::update_payment_modal).post(settings::billing::update_payment),
        )
        .route(
            "/settings/account",
            get(settings::account::page).post(settings::account::action),
        )
        .route("/settings/recipes", get(settings::recipes::page))
        .route("/settings/recipes/create", post(settings::recipes::create))
        .route("/invoices/{id}", get(invoices::detail::page))
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
        .route("/sw.js", get(assets::service_worker))
        .route("/manifest.json", get(assets::manifest))
        .route("/robots.txt", get(assets::robots))
        .route("/sitemap.xml", get(assets::sitemap))
        .nest_service("/static", crate::assets::AssetsService::new())
        .with_state(app_state)
}
