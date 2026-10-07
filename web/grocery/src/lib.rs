//! The aisle-grouped groceries needed by every recipe in the user's list.
//!
//! The logic lives in `imkitchen_web_shared::services::grocery` (shared with
//! the JSON API); this crate only renders its results into templates.

use axum::{
    extract::{Json, State},
    response::{IntoResponse, Redirect},
};
use imkitchen_types::recipe::IngredientUnitFormat;
use serde::Deserialize;
use std::collections::HashSet;

pub use imkitchen_web_shared::services::grocery::{AisleSection, GroceryView, grocery_view};

use imkitchen_web_shared::{
    auth::AuthUser,
    services::grocery,
    state::AppState,
    template::{Template, filters},
};

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/groceries", get(page))
        .route("/groceries/toggle", post(toggle_action))
        // The list itself lives on the kitchen page; `/menu` is kept for old
        // bookmarks (and the calendar-era `/menu/{date}`).
        .route("/menu", get(legacy_menu_redirect))
        .route("/menu/{legacy}", get(legacy_menu_redirect))
}

#[derive(askama::Template)]
#[template(path = "groceries.html")]
pub struct GroceriesTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub recipe_count: usize,
    pub checked: HashSet<String>,
    pub aisles: Vec<AisleSection>,
    /// Index into `aisles` where the right desktop column starts (aisles are
    /// split into two columns balanced by item count).
    pub split_at: usize,
    pub total_items: usize,
    pub checked_items: usize,
    pub progress_pct: usize,
}

impl Default for GroceriesTemplate {
    fn default() -> Self {
        Self {
            current_path: "groceries".to_owned(),
            user: AuthUser::default(),
            recipe_count: 0,
            checked: HashSet::default(),
            aisles: vec![],
            split_at: 0,
            total_items: 0,
            checked_items: 0,
            progress_pct: 0,
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let groceries =
        imkitchen_web_shared::try_page_response!(grocery::load(&app, &user.id), template);
    let view = groceries.view;

    template
        .render(GroceriesTemplate {
            user,
            recipe_count: groceries.recipe_count,
            checked: view.checked,
            aisles: view.aisles,
            split_at: view.split_at,
            total_items: view.total_items,
            checked_items: view.checked_items,
            progress_pct: view.progress_pct,
            ..Default::default()
        })
        .into_response()
}

async fn legacy_menu_redirect() -> impl IntoResponse {
    Redirect::permanent("/")
}

#[derive(Deserialize, Default, Clone)]
pub struct ToggleJson {
    pub name: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn toggle_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Json(input): Json<ToggleJson>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(grocery::toggle(&app, &user.id, input.name), template);

    "<div></div>".into_response()
}
