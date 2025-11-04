use axum::response::IntoResponse;

use crate::extract::template::Template;
use crate::filters;

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

pub async fn page(template: Template<IndexTemplate>) -> impl IntoResponse {
    template.render(IndexTemplate)
}
