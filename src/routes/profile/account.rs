// use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
// use serde::Deserialize;

use crate::auth::AuthUser;
use crate::routes::AppState;
use crate::template::Template;
use crate::template::filters;

#[derive(askama::Template)]
#[template(path = "profile-account.html")]
pub struct AccountTemplate {
    // pub error_message: Option<String>,
    pub current_path: String,
    pub profile_path: String,
    pub user: AuthUser,
}

pub async fn page(template: Template, user: AuthUser) -> impl IntoResponse {
    template.render(AccountTemplate {
        // error_message: None,
        current_path: "profile".to_owned(),
        profile_path: "account".to_owned(),
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
    State(_state): State<AppState>,
    // Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    ""
}
