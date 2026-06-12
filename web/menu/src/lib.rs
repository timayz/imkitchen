use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use imkitchen_core::mealplan::{Generate, Randomize, slot::SlotRow};
use time::OffsetDateTime;

use imkitchen_web_shared::{
    AppState,
    auth::{AuthUser, RequirePremium},
    template::{Status as TemplateStatus, Template, filters},
};

pub struct MenuSlot {
    pub day: u8,
    pub slot: Option<SlotRow>,
}

#[derive(Default, Clone)]
pub struct MenuBoardDay {
    pub date: String,
    pub weekday: String,
    pub day_num: u8,
    pub is_today: bool,
    pub is_past: bool,
    pub is_in_month: bool,
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
    pub selected_day: u64,
    pub current_date: String,
    pub is_past: bool,
    pub first_month_day: u64,
    pub prev_month: String,
    pub next_month: String,
    pub board_weeks: Vec<Vec<MenuBoardDay>>,
    /// Recipe id → slug for every recipe shown, so course cards can link to the
    /// canonical `/r/{slug}` detail page. Missing ids fall back to the id.
    pub slugs: std::collections::HashMap<String, String>,
}

impl MenuTemplate {
    /// Slug for a slot recipe id, falling back to the id when unknown (the
    /// `/r/{param}` route also accepts a raw id).
    pub fn dish_slug<'a>(&'a self, id: &'a str) -> &'a str {
        self.slugs.get(id).map(String::as_str).unwrap_or(id)
    }
}

impl Default for MenuTemplate {
    fn default() -> Self {
        Self {
            current_path: "menu".to_owned(),
            user: AuthUser::default(),
            slots: vec![],
            selected_slot: None,
            selected_day: 0,
            current_date: String::new(),
            is_past: false,
            first_month_day: 0,
            prev_month: "".to_owned(),
            next_month: "".to_owned(),
            board_weeks: vec![],
            slugs: std::collections::HashMap::new(),
        }
    }
}

/// Collects every course recipe id present in a set of slots.
fn slot_recipe_ids(slots: &[SlotRow]) -> Vec<String> {
    let mut ids = vec![];
    for slot in slots {
        ids.push(slot.main_course.id.to_owned());
        for course in [
            &slot.appetizer,
            &slot.accompaniment,
            &slot.dessert,
            &slot.beverage,
            &slot.condiment,
        ]
        .into_iter()
        .flatten()
        {
            ids.push(course.id.to_owned());
        }
    }
    ids
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
    let bounds = imkitchen_web_shared::try_page_response!(sync: bounds, template);
    let (prev_month, next_month) = imkitchen_web_shared::try_page_response!(sync: imkitchen_core::mealplan::prev_next_month(bounds.first), template);
    let slots = imkitchen_web_shared::try_page_response!(
        app.core.mealplan.range(&user.id, bounds.first, bounds.last),
        template
    );

    let slugs = imkitchen_web_shared::try_page_response!(
        app.core.recipe.slugs(slot_recipe_ids(&slots)),
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

    let selected_day = selected_slot.as_ref().map(|s| s.day).unwrap_or(0);

    let today = imkitchen_core::mealplan::now(&user.tz);
    let today_u64 = imkitchen_core::mealplan::date_to_u64(today);
    let is_past = imkitchen_core::mealplan::date_to_u64(bounds.date) < today_u64;

    let fmt = time::macros::format_description!("[year]-[month]-[day]");
    let current_date = bounds.date.format(&fmt).unwrap_or_default();

    // ── Desktop week-board: walk the same Mon–Sun-padded date sequence the
    // calendar uses, but build rich cells (date string for URL, weekday label,
    // slot, today/past flags) and group them into weeks of 7.
    let mut all_board_dates: Vec<OffsetDateTime> =
        imkitchen_core::mealplan::week_days_before(bounds.first);
    let mut d = bounds.first;
    while d <= bounds.last {
        all_board_dates.push(d);
        d += time::Duration::days(1);
    }
    all_board_dates.extend(imkitchen_core::mealplan::week_days_after(bounds.last));

    let bounds_month = bounds.date.month();
    let board_days: Vec<MenuBoardDay> = all_board_dates
        .iter()
        .map(|d| {
            let d_u64 = imkitchen_core::mealplan::date_to_u64(*d);
            let slot = slots
                .iter()
                .find(|s| {
                    OffsetDateTime::from_unix_timestamp(s.day as i64)
                        .map(|sd| imkitchen_core::mealplan::date_to_u64(sd) == d_u64)
                        .unwrap_or(false)
                })
                .cloned();
            MenuBoardDay {
                date: d.format(&fmt).unwrap_or_default(),
                weekday: d.weekday().to_string().chars().take(3).collect(),
                day_num: d.day(),
                is_today: d_u64 == today_u64,
                is_past: d_u64 < today_u64,
                is_in_month: d.month() == bounds_month,
                slot,
            }
        })
        .collect();

    let board_weeks: Vec<Vec<MenuBoardDay>> = board_days.chunks(7).map(|c| c.to_vec()).collect();

    template
        .render(MenuTemplate {
            user,
            slots: menu_slots,
            first_month_day: bounds.first.unix_timestamp() as u64,
            current_date,
            is_past,
            prev_month,
            next_month,
            selected_slot,
            selected_day,
            board_weeks,
            slugs,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn generate_action(
    template: Template,
    State(app): State<AppState>,
    RequirePremium(user): RequirePremium,
    Path((date,)): Path<(String,)>,
) -> impl IntoResponse {
    let preferences = imkitchen_web_shared::try_response!(anyhow:
        app.identity.meal_preferences.load(&user.id),
        template
    );

    let randomize = Some(Randomize {
        cuisine_variety_weight: preferences.cuisine_variety_weight,
        dietary_restrictions: preferences.dietary_restrictions.to_vec(),
    });

    let bounds = imkitchen_web_shared::try_response!(sync anyhow: imkitchen_core::mealplan::month_bounds_from_date(&date, &user.tz), template);
    let now_bounds = imkitchen_web_shared::try_response!(sync anyhow: imkitchen_core::mealplan::month_bounds_from_now(&user.tz), template);
    let (target_local, last_day) = if now_bounds.date > bounds.date {
        (now_bounds.date, now_bounds.last.day())
    } else {
        (bounds.date, bounds.last.day())
    };
    // Use user-tz noon as start so date_to_u64(from_unix_timestamp(start)) yields
    // the user-tz date — from_unix_timestamp always returns UTC, so encoding start
    // at user-tz midnight gives the wrong UTC day for any non-UTC user (e.g. in
    // Martinique UTC-4 evening, midnight rolls into the next UTC day, infinite-polling
    // on a slot stored under tomorrow's date).
    let start_noon = time::PrimitiveDateTime::new(target_local.date(), time::macros::time!(12:00))
        .assume_offset(target_local.offset());
    let start = start_noon.unix_timestamp();
    let days = last_day - target_local.date().day() + 1;

    imkitchen_web_shared::try_response!(
        app.core.mealplan.generate(Generate {
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
    let bounds = imkitchen_web_shared::try_response!(sync anyhow: imkitchen_core::mealplan::month_bounds_from_date(&date, &user.tz), template);
    let now_bounds = imkitchen_web_shared::try_response!(sync anyhow: imkitchen_core::mealplan::month_bounds_from_now(&user.tz), template);

    // Polling must look at the same start day that generate_action used, otherwise
    // we'll keep finding a stale slot from before today (whose generated_at never
    // updates) and never match the aggregate's new generated_at.
    let start = if now_bounds.date > bounds.date {
        now_bounds.date
    } else {
        bounds.date
    };

    let s_generated_at = imkitchen_web_shared::try_response!(anyhow:
        app.core.mealplan.next_slot_from(start, &user.id),
        template,
        Some(GenerateButtonTemplate {
            date,
            status: TemplateStatus::Idle
        })
    )
    .map(|m| m.generated_at);

    let c_generated_at =
        imkitchen_web_shared::try_response!(anyhow: app.core.mealplan.load(&user.id),
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

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::get;
    axum::Router::new()
        .route("/menu", get(page))
        .route("/menu/{date}", get(page))
        .route(
            "/menu/{date}/generate",
            get(generate_modal).post(generate_action),
        )
        .route("/menu/{date}/generate/status", get(generate_status))
}
