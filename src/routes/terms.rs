use axum::response::IntoResponse;

use crate::template::Template;
use crate::template::filters;

#[derive(askama::Template)]
#[template(path = "terms.html")]
pub struct TermsTemplate;

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(TermsTemplate)
}
