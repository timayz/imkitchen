use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Response},
};

use crate::AppState;

#[derive(Template)]
#[template(path = "pages/shopping/current_list.html")]
struct CurrentShoppingListTemplate {
    title: String,
}

/// GET /shopping-lists/current - Current shopping list page
pub async fn current_shopping_list(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = CurrentShoppingListTemplate {
        title: "Current Shopping List".to_string(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /shopping-lists - Shopping lists overview page (redirects to current for now)
pub async fn shopping_lists_overview() -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/shopping-lists/current")
        .body("Redirecting to current shopping list".into())
        .unwrap()
}
