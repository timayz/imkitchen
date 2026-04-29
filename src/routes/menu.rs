use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use imkitchen_core::mealplan::{Generate, Randomize, slot::SlotRow};
use time::OffsetDateTime;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{Status as TemplateStatus, Template, filters},
};

pub struct MenuSlot {
    pub day: u8,
    pub slot: Option<SlotRow>,
}

#[derive(askama::Template)]
#[template(path = "partials/menu-regenerate-modal.html")]
pub struct GenerateModalTemplate {
    pub date: String,
}

#[derive(askama::Template)]
#[template(path = "partials/menu-generate-button.html")]
pub struct GenerateButtonTemplate {
    pub date: String,
    pub status: TemplateStatus,
}

#[derive(askama::Template)]
#[template(path = "menu.html")]
pub struct MenuTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub slots: Vec<MenuSlot>,
    pub selected_slot: Option<SlotRow>,
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
            selected_slot: None,
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
    let bounds = if let Some(Path((date,))) = params {
        imkitchen_core::mealplan::month_bounds_from_date(&date, &user.tz)
    } else {
        imkitchen_core::mealplan::month_bounds_from_now(&user.tz)
    };
    let bounds = crate::try_page_response!(sync: bounds, template);
    let (prev_month, next_month) = crate::try_page_response!(sync: imkitchen_core::mealplan::prev_next_month(bounds.first), template);
    let slots = crate::try_page_response!(
        app.mealplan_query
            .range(&user.id, bounds.first, bounds.last),
        template
    );

    let mut menu_slots = imkitchen_core::mealplan::week_days_before(bounds.first)
        .iter()
        .map(|date| MenuSlot {
            day: date.day(),
            slot: None,
        })
        .collect::<Vec<_>>();

    for day in 1..bounds.last.day() + 1 {
        let slot = slots
            .iter()
            .find_map(|s| {
                let s_day = OffsetDateTime::from_unix_timestamp(s.day as i64)
                    .unwrap()
                    .day();

                if s_day == day {
                    Some(MenuSlot {
                        slot: Some(s.clone()),
                        day: s_day,
                    })
                } else {
                    None
                }
            })
            .unwrap_or(MenuSlot { day, slot: None });

        menu_slots.push(slot);
    }

    for date in imkitchen_core::mealplan::week_days_after(bounds.last) {
        menu_slots.push(MenuSlot {
            day: date.day(),
            slot: None,
        });
    }

    let selected_slot = slots
        .iter()
        .find(|s| {
            let dt = OffsetDateTime::from_unix_timestamp(s.day as i64).unwrap();
            dt.day() == bounds.date.day() && dt.month() == bounds.date.month()
        })
        .or_else(|| slots.first())
        .cloned();

    template
        .render(MenuTemplate {
            user,
            slots: menu_slots,
            first_month_day: bounds.first.unix_timestamp() as u64,
            prev_month,
            next_month,
            selected_slot,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn generate_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((date,)): Path<(String,)>,
) -> impl IntoResponse {
    let preferences = crate::try_response!(anyhow:
        app.identity_cmd.meal_preferences.load(&user.id),
        template
    );

    let randomize = if user.is_premium() {
        Some(Randomize {
            cuisine_variety_weight: preferences.cuisine_variety_weight,
            dietary_restrictions: preferences.dietary_restrictions.to_vec(),
        })
    } else {
        None
    };

    let bounds = crate::try_response!(sync anyhow: imkitchen_core::mealplan::month_bounds_from_date(&date, &user.tz), template);
    let now_bounds = crate::try_response!(sync anyhow: imkitchen_core::mealplan::month_bounds_from_now(&user.tz), template);
    let (start, days) = if now_bounds.date > bounds.date {
        (
            now_bounds.date.unix_timestamp(),
            now_bounds.last.day() - now_bounds.date.day(),
        )
    } else {
        (
            bounds.date.unix_timestamp(),
            bounds.last.day() - bounds.date.day(),
        )
    };

    crate::try_response!(
        app.mealplan_cmd.generate(Generate {
            start: start as u64,
            days,
            user_id: user.id.to_owned(),
            randomize,
            household_size: preferences.household_size,
        }),
        template
    );

    template
        .render(GenerateButtonTemplate {
            date,
            status: TemplateStatus::Pending,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn generate_status(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((date,)): Path<(String,)>,
) -> impl IntoResponse {
    let bounds = crate::try_response!(sync anyhow: imkitchen_core::mealplan::month_bounds_from_date(&date, &user.tz), template);

    let s_generated_at = crate::try_response!(anyhow:
        app.mealplan_query.next_slot_from(bounds.date, &user.id),
        template,
        Some(GenerateButtonTemplate {
            date,
            status: TemplateStatus::Idle
        })
    )
    .map(|m| m.generated_at);

    let c_generated_at = crate::try_response!(anyhow: app.mealplan_cmd.load(&user.id),
        template,
        Some(GenerateButtonTemplate{date, status: TemplateStatus::Idle})
    )
    .map(|m| m.generated_at);

    if s_generated_at == c_generated_at {
        return Redirect::to(&format!("/menu/{date}")).into_response();
    }

    template
        .render(GenerateButtonTemplate {
            date,
            status: TemplateStatus::Checking,
        })
        .into_response()
}

pub async fn generate_modal(
    template: Template,
    Path((date,)): Path<(String,)>,
) -> impl IntoResponse {
    template.render(GenerateModalTemplate { date })
}
