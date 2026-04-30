// use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
// use serde::Deserialize;

use imkitchen_web_shared::AppState;
use imkitchen_web_shared::auth::AuthUser;
use imkitchen_web_shared::template::Template;
use imkitchen_web_shared::template::filters;

#[derive(askama::Template)]
#[template(path = "settings-account.html")]
pub struct SecurityTemplate {
    // pub error_message: Option<String>,
    pub current_path: String,
    pub settings_path: String,
    pub user: AuthUser,
}

pub async fn page(template: Template, user: AuthUser) -> impl IntoResponse {
    template.render(SecurityTemplate {
        // error_message: None,
        current_path: "settings".to_owned(),
        settings_path: "account".to_owned(),
        user,
    })
}
//
// #[derive(Deserialize)]
// pub struct ActionInput {
//     pub email: String,
// }

pub async fn action(
    _template: Template,
    State(_app): State<AppState>,
    // Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    ""
}
