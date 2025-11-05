use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::filters;
use crate::server::AppState;
use crate::template::Template;

#[derive(askama::Template)]
#[template(path = "profile-security.html")]
pub struct SecurityTemplate {
    pub error_message: Option<String>,
    pub current_path: String,
    pub profile_path: String,
}

pub async fn page(template: Template<SecurityTemplate>) -> impl IntoResponse {
    template.render(SecurityTemplate {
        error_message: None,
        current_path: "profile".to_owned(),
        profile_path: "security".to_owned(),
    })
}

#[derive(Deserialize)]
pub struct ActionInput {
    pub email: String,
}

pub async fn action(
    template: Template<SecurityTemplate>,
    State(state): State<AppState>,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    ""
}
