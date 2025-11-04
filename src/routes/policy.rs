use axum::response::IntoResponse;

use crate::extract::template::Template;

#[derive(askama::Template)]
#[template(path = "policy.html")]
pub struct PolicyTemplate;

pub async fn page(template: Template<PolicyTemplate>) -> impl IntoResponse {
    template.render(PolicyTemplate)
}
