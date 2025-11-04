use axum::response::IntoResponse;

use crate::extract::template::Template;

#[derive(askama::Template)]
#[template(path = "terms.html")]
pub struct TermsTemplate;

pub async fn page(template: Template<TermsTemplate>) -> impl IntoResponse {
    template.render(TermsTemplate)
}
