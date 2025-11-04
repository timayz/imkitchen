use axum::response::IntoResponse;

use crate::extract::template::Template;
use crate::filters;

#[derive(askama::Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate;

pub async fn page(template: Template<RegisterTemplate>) -> impl IntoResponse {
    template.render(RegisterTemplate)
}
