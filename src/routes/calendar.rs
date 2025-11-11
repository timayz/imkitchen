use axum::response::IntoResponse;

use crate::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "coming-soon.html")]
pub struct CalendarTemplate;

pub async fn page(template: Template<CalendarTemplate>) -> impl IntoResponse {
    template.render(CalendarTemplate)
}
