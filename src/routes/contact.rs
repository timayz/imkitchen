use axum::response::IntoResponse;

use crate::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "contact.html")]
pub struct ContactTemplate;

pub async fn page(template: Template<ContactTemplate>) -> impl IntoResponse {
    template.render(ContactTemplate)
}
