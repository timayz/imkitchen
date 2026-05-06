use axum::response::IntoResponse;

use imkitchen_web_shared::template::Template;
use imkitchen_web_shared::template::filters;

#[derive(askama::Template)]
#[template(path = "legal.html")]
pub struct LegalTemplate;

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(LegalTemplate)
}
