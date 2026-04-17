use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use imkitchen_mealplan::{
    Generate, Randomize,
    slot::SlotRow,
    week::{WeekListRow, WeekRow},
};
use time::OffsetDateTime;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{Status as TemplateStatus, Template, filters},
};

pub enum MenuSlot {
    Day(u8),
    Slot(SlotRow, u8),
}

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
pub struct MenuTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub slots: Vec<MenuSlot>,
    pub first_month_day: u64,
    pub prev_month: String,
    pub next_month: String,
}

impl Default for MenuTemplate {
    fn default() -> Self {
        Self {
            current_path: "menu".to_owned(),
            user: AuthUser::default(),
            slots: vec![],
            first_month_day: 0,
            prev_month: "".to_owned(),
            next_month: "".to_owned(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    params: Option<Path<(String,)>>,
) -> impl IntoResponse {
    let bounds = crate::try_page_response!(sync: imkitchen_mealplan::month_bounds_from_now(&user.tz), template);
    let (prev_month, next_month) = crate::try_page_response!(sync: imkitchen_mealplan::prev_next_month(bounds.first), template);
    let slots = crate::try_page_response!(
        app.mealplan_query
            .range(&user.id, bounds.first, bounds.last),
        template
    );

    let mut menu_slots = imkitchen_mealplan::week_days_before(bounds.first)
        .iter()
        .map(|date| MenuSlot::Day(date.day()))
        .collect::<Vec<_>>();

    for day in 1..bounds.last.day() + 1 {
        let slot = slots
            .iter()
            .find_map(|s| {
                let s_day = OffsetDateTime::from_unix_timestamp(s.day as i64)
                    .unwrap()
                    .day();

                if s_day == day {
                    Some(MenuSlot::Slot(s.clone(), s_day))
                } else {
                    None
                }
            })
            .unwrap_or(MenuSlot::Day(day));

        menu_slots.push(slot);
    }

    for date in imkitchen_mealplan::week_days_after(bounds.last) {
        menu_slots.push(MenuSlot::Day(date.day()));
    }

    template
        .render(MenuTemplate {
            user,
            slots: menu_slots,
            first_month_day: bounds.first.unix_timestamp() as u64,
            prev_month,
            next_month,
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
