use axum::response::IntoResponse;

use crate::filters;
use crate::template::Template;

#[derive(askama::Template)]
#[template(path = "help.html")]
pub struct HelpTemplate;

pub async fn page(template: Template<HelpTemplate>) -> impl IntoResponse {
    template.render(HelpTemplate)
}
