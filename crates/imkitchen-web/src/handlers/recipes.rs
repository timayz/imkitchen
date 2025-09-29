use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Response},
};

use crate::AppState;

#[derive(Template)]
#[template(path = "pages/recipes/discover.html")]
struct RecipeDiscoverTemplate {
    title: String,
}

/// GET /recipes/discover - Recipe discovery page
pub async fn recipe_discover(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = RecipeDiscoverTemplate {
        title: "Discover Recipes".to_string(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /recipes - Recipes overview page (redirects to discover for now)
pub async fn recipes_overview() -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/recipes/discover")
        .body("Redirecting to recipe discovery".into())
        .unwrap()
}
