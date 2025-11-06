use axum::response::IntoResponse;

use crate::template::Template;

#[derive(askama::Template)]
#[template(path = "admin-users.html")]
pub struct UsersTemplate {
    pub current_path: String,
}

pub async fn page(template: Template<UsersTemplate>) -> impl IntoResponse {
    template.render(UsersTemplate {
        current_path: "users".to_owned(),
    })
}
