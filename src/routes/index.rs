use axum::extract::State;
use axum::response::IntoResponse;
use imkitchen_mealplan::SlotRow;

use crate::auth::AuthOptional;
use crate::routes::AppState;
use crate::template::{ServerErrorTemplate, Template, filters};

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

#[derive(askama::Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
    pub slot: Option<SlotRow>,
}

impl Default for DashboardTemplate {
    fn default() -> Self {
        Self {
            current_path: "dashboard".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            slot: None,
        }
    }
}

pub async fn page(
    template: Template<IndexTemplate>,
    server_error: Template<ServerErrorTemplate>,
    dashboard: Template<DashboardTemplate>,
    AuthOptional(user): AuthOptional,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let Some(user) = user else {
        return template.render(IndexTemplate).into_response();
    };

    let day = imkitchen_mealplan::weekday_from_now();
    let slot = match app.mealplan_query.next_slot_from(day, &user.id).await {
        Ok(slot) => slot,
        Err(err) => {
            tracing::error!(user = user.id, err = %err, "failed to find slot");

            return server_error.render(ServerErrorTemplate).into_response();
        }
    };

    dashboard
        .render(DashboardTemplate {
            user,
            slot,
            ..Default::default()
        })
        .into_response()
}

// @TODO: if on last week generated, add html element on dashboard page that will call action to
// generate mealplan for next four weeks. Use MealPlanLastWeek to detect last week
