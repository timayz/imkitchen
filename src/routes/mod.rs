use axum::response::IntoResponse;

use crate::filters;
use crate::template::Template;

pub mod health;
pub mod help;
pub mod index;
pub mod login;
pub mod policy;
pub mod profile;
pub mod register;
pub mod service_worker;
pub mod terms;

#[derive(askama::Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate;

pub async fn fallback(template: Template<NotFoundTemplate>) -> impl IntoResponse {
    template.render(NotFoundTemplate)
}
