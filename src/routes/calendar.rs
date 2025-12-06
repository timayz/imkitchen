use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use imkitchen_mealplan::{Status, WeekListRow, WeekRow};
use imkitchen_shared::Metadata;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{SERVER_ERROR_MESSAGE, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "calendar-regenerate-modal.html")]
pub struct RegenerateModalTemplate;

#[derive(askama::Template)]
#[template(path = "calendar-regenerate-status.html")]
pub struct RegenerateStatusTemplate;

#[derive(askama::Template)]
#[template(path = "calendar-regenerate.html")]
pub struct RegenerateTemplate {
    pub error_message: Option<String>,
}

#[derive(askama::Template)]
#[template(path = "calendar.html")]
pub struct CalendarTemplate {
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
    pub weeks: Vec<WeekListRow>,
    pub weekday: u64,
    pub current: Option<WeekRow>,
    pub index: u8,
}

impl Default for CalendarTemplate {
    fn default() -> Self {
        Self {
            current_path: "calendar".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            weeks: Default::default(),
            current: None,
            weekday: 0,
            index: 0,
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    AuthUser(user): AuthUser,
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

    template
        .render(CalendarTemplate {
            user,
            weeks,
            current,
            weekday,
            index,
            ..Default::default()
        })
        .into_response()
}

pub async fn regenerate_action(
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let status = match app.mealplan_command.load_optional(&user.id).await {
        Ok(Some(r)) => r.item.status,
        Ok(_) => Status::Idle,
        Err(err) => {
            tracing::error!(user = user.id, err = %err,"Failed to regenerate meal plan");

            return template.render(RegenerateTemplate {
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
            });
        }
    };

    if status == Status::Processing {
        return template.render(RegenerateTemplate {
            error_message: Some("Invalid status".to_owned()),
        });
    }

    match app
        .mealplan_command
        .generate(&Metadata::by(user.id.to_owned()))
        .await
    {
        Ok(_) => template.render(RegenerateTemplate {
            error_message: None,
        }),
        Err(err) => {
            tracing::error!(user = user.id, err = %err, "Failed to regenerate meal plan");

            template.render(RegenerateTemplate {
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
            })
        }
    }
}

pub async fn regenerate_status(
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let meal_plan = match app.mealplan_command.load_optional(&user.id).await {
        Ok(Some(loaded)) => loaded,
        Ok(_) => return template.render(RegenerateStatusTemplate).into_response(),
        Err(err) => {
            tracing::error!(user = user.id, err = %err,"Failed to check meal plan regeneration status");

            return Redirect::to("/calendar/week-1").into_response();
        }
    };

    if meal_plan.item.status == Status::Failed {
        return Redirect::to("/calendar/week-1").into_response();
    }

    let week_from_now = imkitchen_mealplan::next_four_mondays_from_now()[0];

    let week = match app.mealplan_query.find(week_from_now.start, &user.id).await {
        Ok(Some(week)) => week,
        Ok(_) => return template.render(RegenerateStatusTemplate).into_response(),
        Err(err) => {
            tracing::error!(user = user.id, err = %err,"Failed to check meal plan regeneration status");

            return Redirect::to("/calendar/week-1").into_response();
        }
    };

    if week.status.0 == Status::Processing {
        return template.render(RegenerateStatusTemplate).into_response();
    }

    Redirect::to("/calendar/week-1").into_response()
}

pub async fn regenerate_modal(template: Template) -> impl IntoResponse {
    template.render(RegenerateModalTemplate)
}
