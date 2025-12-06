use axum::response::IntoResponse;

use crate::template::Template;
use crate::template::filters;

#[derive(askama::Template)]
#[template(path = "help.html")]
pub struct HelpTemplate;

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(HelpTemplate)
}
