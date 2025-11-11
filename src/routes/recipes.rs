use axum::response::IntoResponse;

use crate::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "coming-soon.html")]
pub struct RecipesTemplate;

pub async fn page(template: Template<RecipesTemplate>) -> impl IntoResponse {
    template.render(RecipesTemplate)
}
