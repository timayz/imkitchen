// use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
// use serde::Deserialize;

use crate::routes::AppState;
use crate::template::Template;
use crate::template::filters;

#[derive(askama::Template)]
#[template(path = "profile-notifications.html")]
pub struct NotificationsTemplate {
    // pub error_message: Option<String>,
    pub current_path: String,
    pub profile_path: String,
}

pub async fn page(template: Template<NotificationsTemplate>) -> impl IntoResponse {
    template.render(NotificationsTemplate {
        // error_message: None,
        current_path: "profile".to_owned(),
        profile_path: "notifications".to_owned(),
    })
}
//
// #[derive(Deserialize)]
// pub struct ActionInput {
//     pub email: String,
// }

pub async fn action(
    _template: Template<NotificationsTemplate>,
    State(_app): State<AppState>,
    // Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    ""
}
