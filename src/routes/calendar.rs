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
        imkitchen_mealplan::week::filter_last_from(&app.read_db, week_from_now.start, &user.id),
        template
    );

    if index == 0 {
        index += 1;
    }

    let current = match weeks.get((index - 1) as usize) {
        Some(week) => crate::try_page_response!(
            imkitchen_mealplan::week::find_from_unix_timestamp(&app.read_db, week.start, &user.id),
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
        !crate::try_page_response!(
            imkitchen_mealplan::first_week_recipes(
                &app.read_db,
                &user.id,
                imkitchen_recipe::RecipeType::MainCourse
            ),
            template
        )
        .is_empty()
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
    let preferences = crate::try_response!(anyhow:
        imkitchen_user::meal_preferences::load(&app.executor, &user.id),
        template
    );
    let weeks = imkitchen_mealplan::next_four_mondays_from_now()
        .iter()
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
        imkitchen_mealplan::Command::generate(
            &app.executor,
            &app.read_db,
            Generate {
                weeks,
                user_id: user.id.to_owned(),
                randomize,
                household_size: preferences.household_size,
            }
        ),
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
    let week_from_now = imkitchen_mealplan::next_four_mondays_from_now()[0];

    match crate::try_response!(anyhow:
        imkitchen_mealplan::week::find(&app.read_db,week_from_now.start, &user.id),
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
