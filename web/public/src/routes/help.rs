use axum::response::IntoResponse;

use imkitchen_web_shared::template::Template;
use imkitchen_web_shared::template::filters;

#[derive(askama::Template)]
#[template(path = "help.html")]
pub struct HelpTemplate;

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(HelpTemplate)
}
