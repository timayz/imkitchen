use axum::response::IntoResponse;

use crate::auth::AuthOptional;
use crate::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

#[derive(askama::Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
}

impl Default for DashboardTemplate {
    fn default() -> Self {
        Self {
            current_path: "dashboard".to_owned(),
            user: imkitchen_user::AuthUser::default(),
        }
    }
}

pub async fn page(
    template: Template<IndexTemplate>,
    dashboard: Template<DashboardTemplate>,
    AuthOptional(user): AuthOptional,
) -> impl IntoResponse {
    let Some(user) = user else {
        return template.render(IndexTemplate).into_response();
    };

    dashboard
        .render(DashboardTemplate {
            user,
            ..Default::default()
        })
        .into_response()
}

// @TODO: if on last week generated, add html element on dashboard page that will call action to
// generate mealplan for next four weeks. Use MealPlanLastWeek to detect last week
