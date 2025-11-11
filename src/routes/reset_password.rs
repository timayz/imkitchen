use axum::response::IntoResponse;

use crate::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "coming-soon.html")]
pub struct ResetPasswordTemplate;

pub async fn page(template: Template<ResetPasswordTemplate>) -> impl IntoResponse {
    template.render(ResetPasswordTemplate)
}
