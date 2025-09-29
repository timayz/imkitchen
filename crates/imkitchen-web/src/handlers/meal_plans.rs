use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Response},
};

use crate::AppState;

#[derive(Template)]
#[template(path = "pages/meal_plans/current.html")]
struct CurrentMealPlanTemplate {
    title: String,
    current_page_title: String,
}

/// GET /meal-plans/current - Current meal plan page
pub async fn current_meal_plan(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = CurrentMealPlanTemplate {
        title: "Current Meal Plan".to_string(),
        current_page_title: "Current Meal Plan".to_string(),
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /meal-plans - Meal plans overview page (redirects to current for now)
pub async fn meal_plans_overview() -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/meal-plans/current")
        .body("Redirecting to current meal plan".into())
        .unwrap()
}
