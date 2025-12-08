use axum::{http::header, response::IntoResponse};

use crate::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "manifest.json")]
pub struct ManisfestTemplate;

pub async fn asset(template: Template) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
        ],
        template.render(ManisfestTemplate),
    )
}
