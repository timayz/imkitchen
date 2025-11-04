use axum::response::IntoResponse;

use crate::extract::template::Template;
use crate::filters;

#[derive(askama::Template)]
#[template(path = "login.html")]
pub struct LoginTemplate;

pub async fn page(template: Template<LoginTemplate>) -> impl IntoResponse {
    template.render(LoginTemplate)
}
