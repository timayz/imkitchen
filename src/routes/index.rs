use axum::extract::State;
use axum::response::IntoResponse;
use imkitchen_mealplan::{DaySlotRecipe, SlotRow, WeekRow};

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
    pub week: Option<WeekRow>,
    pub prep_remiders: Option<Vec<DaySlotRecipe>>,
    pub generate_next_weeks_needed: bool,
}

impl Default for DashboardTemplate {
    fn default() -> Self {
        Self {
            current_path: "dashboard".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            slot: None,
            week: None,
            prep_remiders: None,
            generate_next_weeks_needed: false,
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
    let prep_remiders = if let Some(ref slot) = slot {
        match app
            .mealplan_query
            .next_prep_remiders_from(slot.day, &user.id)
            .await
        {
            Ok(remiders) => remiders,
            Err(err) => {
                tracing::error!(user = user.id, err = %err, "failed to find next slot");

                return server_error.render(ServerErrorTemplate).into_response();
            }
        }
    } else {
        None
    };

    let week_from_now = imkitchen_mealplan::current_and_next_four_weeks_from_now()[0];
    let week = match app
        .mealplan_query
        .find_last_from(week_from_now.start, &user.id)
        .await
    {
        Ok(week) => week,
        Err(err) => {
            tracing::error!(
                user = user.id,
                err = %err,
                "failed to get find last week on dashboard page"
            );

            return server_error.render(ServerErrorTemplate).into_response();
        }
    };

    let generate_next_weeks_needed = match (
        week.as_ref(),
        app.mealplan_command.find_last_week(&user.id).await,
    ) {
        (Some(week), Ok(Some(last_week))) => week.start == last_week,
        (_, Err(err)) => {
            tracing::error!(
                user = user.id,
                err = %err,
                "failed to get find last week on dashboard page"
            );

            return server_error.render(ServerErrorTemplate).into_response();
        }
        _ => false,
    };

    dashboard
        .render(DashboardTemplate {
            user,
            slot,
            week,
            prep_remiders,
            generate_next_weeks_needed,
            ..Default::default()
        })
        .into_response()
}
