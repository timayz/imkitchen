// use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
// use serde::Deserialize;

use crate::auth::AuthUser;
use crate::routes::AppState;
use crate::template::Template;
use crate::template::filters;

#[derive(askama::Template)]
#[template(path = "profile-security.html")]
pub struct SecurityTemplate {
    // pub error_message: Option<String>,
    pub current_path: String,
    pub profile_path: String,
    pub user: imkitchen_user::AuthUser,
}

pub async fn page(
    template: Template<SecurityTemplate>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    template.render(SecurityTemplate {
        // error_message: None,
        current_path: "profile".to_owned(),
        profile_path: "security".to_owned(),
        user,
    })
}
//
// #[derive(Deserialize)]
// pub struct ActionInput {
//     pub email: String,
// }

pub async fn action(
    _template: Template<SecurityTemplate>,
    State(_app): State<AppState>,
    // Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    ""
}
