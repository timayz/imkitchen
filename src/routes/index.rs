use axum::extract::State;
use axum::http::request::Parts;
use axum::response::IntoResponse;
use imkitchen_mealplan::{DaySlotRecipe, SlotRow, WeekRow};

use crate::auth::AuthOptional;
use crate::routes::AppState;
use crate::template::{Template, filters};

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub show_nav: bool,
}

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

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn page(
    template: Template<IndexTemplate>,
    dashboard: Template<DashboardTemplate>,
    AuthOptional(user): AuthOptional,
    State(app): State<AppState>,
    mut parts: Parts,
) -> impl IntoResponse {
    crate::template::into_page_response(
        page_handler(template, dashboard, user, &app).await,
        &mut parts,
        &app,
    )
    .await
}

pub async fn page_handler(
    template: Template<IndexTemplate>,
    dashboard: Template<DashboardTemplate>,
    user: Option<imkitchen_user::AuthUser>,
    app: &AppState,
) -> imkitchen_shared::Result<Option<impl IntoResponse + use<>>> {
    let Some(user) = user else {
        return Ok(Some(
            template
                .render(IndexTemplate { show_nav: true })
                .into_response(),
        ));
    };

    tracing::Span::current().record("user", &user.id);

    let day = imkitchen_mealplan::weekday_from_now();
    let slot = app.mealplan_query.next_slot_from(day, &user.id).await?;
    let prep_remiders = if let Some(ref slot) = slot {
        app.mealplan_query
            .next_prep_remiders_from(slot.day, &user.id)
            .await?
    } else {
        None
    };

    let week_from_now = imkitchen_mealplan::current_and_next_four_weeks_from_now()[0];
    let week = app
        .mealplan_query
        .find_last_from(week_from_now.start, &user.id)
        .await?;
    let last_week = app.mealplan_command.find_last_week(&user.id).await?;

    let generate_next_weeks_needed = match (week.as_ref(), last_week) {
        (Some(week), Some(last_week)) => week.start == last_week,
        _ => false,
    };

    Ok(Some(
        dashboard
            .render(DashboardTemplate {
                user,
                slot,
                week,
                prep_remiders,
                generate_next_weeks_needed,
                ..Default::default()
            })
            .into_response(),
    ))
}
