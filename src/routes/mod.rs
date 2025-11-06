use axum::response::IntoResponse;

use crate::template::{NotFoundTemplate, Template};

pub mod health;
pub mod help;
pub mod index;
pub mod login;
pub mod policy;
pub mod profile;
pub mod register;
pub mod service_worker;
pub mod terms;

pub async fn fallback(template: Template<NotFoundTemplate>) -> impl IntoResponse {
    template.render(NotFoundTemplate)
}
