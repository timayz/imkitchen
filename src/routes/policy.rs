use axum::response::IntoResponse;

use crate::template::Template;
use crate::template::filters;

#[derive(askama::Template)]
#[template(path = "policy.html")]
pub struct PolicyTemplate;

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(PolicyTemplate)
}
