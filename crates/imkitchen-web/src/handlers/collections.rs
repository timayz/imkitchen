use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};

use crate::AppState;

#[derive(Template)]
#[template(path = "pages/collections/index.html")]
struct CollectionsTemplate {
    title: String,
}

/// GET /collections - Collections page
pub async fn collections_index(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = CollectionsTemplate {
        title: "My Collections".to_string(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
