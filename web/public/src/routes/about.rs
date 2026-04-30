use axum::response::IntoResponse;

use imkitchen_web_shared::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub show_nav: bool,
}

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(IndexTemplate { show_nav: false })
}
