use axum::response::IntoResponse;

use crate::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "coming-soon.html")]
pub struct CommunityTemplate;

pub async fn page(template: Template<CommunityTemplate>) -> impl IntoResponse {
    template.render(CommunityTemplate)
}
