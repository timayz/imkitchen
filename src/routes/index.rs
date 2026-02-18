use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use imkitchen_mealplan::slot::SlotRow;
use imkitchen_mealplan::week::WeekRow;
use imkitchen_shared::{mealplan::DaySlotRecipe, recipe::RecipeType};

use crate::auth::{AuthToken, AuthUser};
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
    pub user: AuthUser,
    pub slot: Option<SlotRow>,
    pub slot_recipe: Option<imkitchen_recipe::query::user::UserView>,
    pub slot_completed_count: u8,
    pub slot_total_count: u8,
    pub week: Option<WeekRow>,
    pub prep_remiders: Option<Vec<DaySlotRecipe>>,
    pub generate_next_weeks_needed: bool,
}

impl Default for DashboardTemplate {
    fn default() -> Self {
        Self {
            current_path: "dashboard".to_owned(),
            user: AuthUser::default(),
            slot: None,
            slot_recipe: None,
            week: None,
            prep_remiders: None,
            generate_next_weeks_needed: false,
            slot_completed_count: 0,
            slot_total_count: 1,
        }
    }
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn page(
    template: Template,
    user: Option<AuthUser>,
    token: Option<AuthToken>,
    State(app): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let (Some(user), Some(token)) = (user, token) else {
        return template
            .render(IndexTemplate { show_nav: true })
            .into_response();
    };

    tracing::Span::current().record("user", &user.id);

    let day = imkitchen_mealplan::now(&user.tz);
    let slot =
        crate::try_page_response!(app.mealplan_query.next_slot_from(day, &user.id), template);

    let mut slot_completed_count = 0;
    let mut slot_total_count = 1;
    let mut slot_recipe = None;

    if let Some(ref slot) = slot {
        let mut slot_recipe_id = None;

        if let Some(ref appetizer) = slot.appetizer {
            slot_total_count += 1;

            if appetizer.is_completed() {
                slot_completed_count += 1;
            } else if slot_recipe_id.is_none() {
                slot_recipe_id = Some(&appetizer.id);
            }
        }

        if slot.main_course.is_completed() {
            slot_completed_count += 1;
        } else if slot_recipe_id.is_none() {
            slot_recipe_id = Some(&slot.main_course.id);
        }

        if let Some(ref accompaniment) = slot.accompaniment {
            slot_total_count += 1;

            if accompaniment.is_completed() {
                slot_completed_count += 1;
            } else if slot_recipe_id.is_none() {
                slot_recipe_id = Some(&accompaniment.id);
            }
        }

        if let Some(ref dessert) = slot.dessert {
            slot_total_count += 1;

            if dessert.is_completed() {
                slot_completed_count += 1;
            } else if slot_recipe_id.is_none() {
                slot_recipe_id = Some(&dessert.id);
            }
        }

        slot_recipe = crate::try_page_response!(
            app.recipe_query
                .find_user(slot_recipe_id.unwrap_or(&slot.main_course.id)),
            template
        );
    };

    let prep_remiders = if let Some(ref slot) = slot {
        crate::try_page_response!(
            app.mealplan_query
                .next_prep_remiders_from(slot.day, &user.id),
            template
        )
    } else {
        None
    };

    let week_from_now = imkitchen_mealplan::current_and_next_four_weeks_from_now(&user.tz)[0];
    let week = crate::try_page_response!(
        app.mealplan_query
            .find_week_last_from(week_from_now.start, &user.id),
        template
    );
    let last_week = crate::try_page_response!(app.mealplan_query.last_week(&user.id), template);

    let generate_next_weeks_needed = match (week.as_ref(), last_week) {
        (Some(week), Some(last_week)) => week.start == last_week.week,
        (_, Some(_)) => true,
        _ => false,
    };

    let auth_cookie = crate::try_page_response!(sync:
        crate::auth::build_cookie(app.config.jwt, token.sub.to_owned(), token.acc.to_owned()),
        template
    );

    let jar = jar.add(auth_cookie);

    (
        jar,
        template.render(DashboardTemplate {
            user,
            slot,
            slot_recipe,
            week,
            prep_remiders,
            generate_next_weeks_needed,
            slot_total_count,
            slot_completed_count,
            ..Default::default()
        }),
    )
        .into_response()
}
