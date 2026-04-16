use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use imkitchen_mealplan::{
    Generate, Randomize,
    week::{WeekListRow, WeekRow},
};

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{Status as TemplateStatus, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "partials/calendar-regenerate-modal.html")]
pub struct RegenerateModalTemplate;

#[derive(askama::Template)]
#[template(path = "partials/calendar-generate-button.html")]
pub struct RegenerateButtonTemplate {
    pub status: TemplateStatus,
}

#[derive(askama::Template)]
#[template(path = "menu.html")]
pub struct CalendarTemplate {
    pub current_path: String,
    pub user: AuthUser,
}

impl Default for CalendarTemplate {
    fn default() -> Self {
        Self {
            current_path: "menu".to_owned(),
            user: AuthUser::default(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    params: Option<Path<(u16, u8)>>,
) -> impl IntoResponse {
    template
        .render(CalendarTemplate {
            user,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn regenerate_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let preferences = crate::try_response!(anyhow:
        app.user_cmd.meal_preferences.load(&user.id),
        template
    );
    let weeks = imkitchen_mealplan::current_and_next_four_weeks_from_now(&user.tz);
    let last_week = crate::try_page_response!(
        app.mealplan_query
            .find_week_last_from(weeks[0].start, &user.id),
        template
    );
    let skip_n = if last_week.is_some() { 1 } else { 0 };
    let weeks = weeks
        .iter()
        .skip(skip_n)
        .map(|w| {
            (
                w.start.unix_timestamp() as u64,
                w.end.unix_timestamp() as u64,
            )
        })
        .collect::<Vec<_>>();

    let randomize = if user.is_premium() {
        Some(Randomize {
            cuisine_variety_weight: preferences.cuisine_variety_weight,
            dietary_restrictions: preferences.dietary_restrictions.to_vec(),
        })
    } else {
        None
    };

    crate::try_response!(
        app.mealplan_cmd.generate(Generate {
            weeks,
            user_id: user.id.to_owned(),
            randomize,
            household_size: preferences.household_size,
        }),
        template
    );

    template
        .render(RegenerateButtonTemplate {
            status: TemplateStatus::Pending,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn regenerate_status(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let week_from_now = imkitchen_mealplan::next_four_mondays_from_now(&user.tz)[0];

    match crate::try_response!(anyhow:
        app.mealplan_query.find_week(week_from_now.start, &user.id),
        template,
        Some(RegenerateButtonTemplate {
            status: TemplateStatus::Idle
        })
    ) {
        Some(week) => week,
        _ => {
            return template
                .render(RegenerateButtonTemplate {
                    status: TemplateStatus::Checking,
                })
                .into_response();
        }
    };

    Redirect::to("/calendar/week-1").into_response()
}

pub async fn regenerate_modal(template: Template) -> impl IntoResponse {
    template.render(RegenerateModalTemplate)
}
