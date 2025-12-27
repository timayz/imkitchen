use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use imkitchen_mealplan::{Status, WeekListRow, WeekRow};
use imkitchen_shared::Metadata;

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
#[template(path = "calendar.html")]
pub struct CalendarTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub weeks: Vec<WeekListRow>,
    pub weekday: u64,
    pub current: Option<WeekRow>,
    pub index: u8,
    pub is_empty_state: bool,
    pub generation_needed: bool,
}

impl Default for CalendarTemplate {
    fn default() -> Self {
        Self {
            current_path: "calendar".to_owned(),
            user: AuthUser::default(),
            weeks: Default::default(),
            current: None,
            weekday: 0,
            index: 0,
            is_empty_state: false,
            generation_needed: false,
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((mut index,)): Path<(u8,)>,
) -> impl IntoResponse {
    let week_from_now = imkitchen_mealplan::current_and_next_four_weeks_from_now()[0];
    let weeks = crate::try_page_response!(
        app.mealplan_query
            .filter_last_from(week_from_now.start, &user.id),
        template
    );

    if index == 0 {
        index += 1;
    }

    let current = match weeks.get((index - 1) as usize) {
        Some(week) => crate::try_page_response!(
            app.mealplan_query
                .find_from_unix_timestamp(week.start, &user.id),
            template
        ),
        _ => None,
    };

    let weekday = imkitchen_mealplan::weekday_from_now().unix_timestamp() as u64;

    let is_empty_state = current
        .as_ref()
        .map(|c| c.slots.is_empty())
        .unwrap_or(weeks.is_empty());

    let generation_needed = if is_empty_state {
        crate::try_page_response!(
            app.mealplan_command
                .has(&user.id, imkitchen_recipe::RecipeType::MainCourse),
            template
        )
    } else {
        false
    };

    template
        .render(CalendarTemplate {
            user,
            weeks,
            current,
            weekday,
            index,
            is_empty_state,
            generation_needed,
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
    let status =
        crate::try_response!(anyhow: app.mealplan_command.load_optional(&user.id), template)
            .map(|r| r.item.status)
            .unwrap_or(Status::Idle);

    if status == Status::Processing {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::User(
                "Mealplan already generating".to_owned()
            )),
            template
        );
    }

    crate::try_response!(
        app.mealplan_command
            .generate(user.is_premium(), &Metadata::by(user.id.to_owned())),
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
    let meal_plan = match crate::try_response!(anyhow: app.mealplan_command.load_optional(&user.id), template)
    {
        Some(loaded) => loaded,
        _ => {
            return template
                .render(RegenerateButtonTemplate {
                    status: TemplateStatus::Checking,
                })
                .into_response();
        }
    };

    if meal_plan.item.status == Status::Failed {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::User(
                meal_plan.item.reason.unwrap_or_default()
            )),
            template,
            Some(RegenerateButtonTemplate {status: TemplateStatus::Idle})
        );
    }

    let week_from_now = imkitchen_mealplan::next_four_mondays_from_now()[0];

    let week = match crate::try_response!(anyhow:
        app.mealplan_query.find(week_from_now.start, &user.id),
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

    if week.status.0 == Status::Processing {
        return template
            .render(RegenerateButtonTemplate {
                status: TemplateStatus::Checking,
            })
            .into_response();
    }

    Redirect::to("/calendar/week-1").into_response()
}

pub async fn regenerate_modal(template: Template) -> impl IntoResponse {
    template.render(RegenerateModalTemplate)
}
