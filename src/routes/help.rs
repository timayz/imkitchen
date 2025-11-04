use axum::response::IntoResponse;

use crate::extract::template::Template;
use crate::filters;

#[derive(askama::Template)]
#[template(path = "help.html")]
pub struct HelpTemplate;

pub async fn page(template: Template<HelpTemplate>) -> impl IntoResponse {
    template.render(HelpTemplate)
}
