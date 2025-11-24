use axum::response::IntoResponse;

use crate::template::Template;
use crate::template::filters;

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

pub async fn page(template: Template<IndexTemplate>) -> impl IntoResponse {
    template.render(IndexTemplate)
}

// @TODO: if on last week generated, add html element on dashboard page that will call action to
// generate mealplan for next four weeks. Use MealPlanLastWeek to detect last week
