use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::filters;
use crate::server::AppState;
use crate::template::Template;

#[derive(askama::Template)]
#[template(path = "profile-meal-preferences.html")]
pub struct MealPreferencesTemplate {
    pub error_message: Option<String>,
    pub current_path: String,
    pub profile_path: String,
}

pub async fn page(template: Template<MealPreferencesTemplate>) -> impl IntoResponse {
    template.render(MealPreferencesTemplate {
        error_message: None,
        current_path: "profile".to_owned(),
        profile_path: "meal-preferences".to_owned(),
    })
}

#[derive(Deserialize)]
pub struct ActionInput {
    pub email: String,
}

pub async fn action(
    template: Template<MealPreferencesTemplate>,
    State(state): State<AppState>,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    ""
}
