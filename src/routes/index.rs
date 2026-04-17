use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use imkitchen_mealplan::ChangeSlotRecipeStatus;
use imkitchen_mealplan::slot::SlotRow;
use imkitchen_mealplan::week::WeekRow;
use imkitchen_shared::mealplan::DaySlotStatus;
use imkitchen_shared::recipe::{IngredientUnitFormat, Instruction};
use imkitchen_shared::{mealplan::DaySlotRecipe, recipe::RecipeType};

use crate::auth::{AuthToken, AuthUser};
use crate::routes::AppState;
use crate::template::{NotFoundTemplate, Template, filters};

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub show_nav: bool,
}

#[derive(askama::Template)]
#[template(path = "kitchen.html")]
pub struct KitchenTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub slot: Option<SlotRow>,
    pub slot_recipe: Option<imkitchen_recipe::query::user::UserView>,
    pub slot_completed_count: u8,
    pub slot_total_count: u8,
    pub week: Option<WeekRow>,
    pub prep_remiders: Option<Vec<DaySlotRecipe>>,
    pub generate_next_weeks_needed: bool,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub date: String,
}

impl Default for KitchenTemplate {
    fn default() -> Self {
        Self {
            current_path: "kitchen".to_owned(),
            user: AuthUser::default(),
            slot: None,
            slot_recipe: None,
            week: None,
            prep_remiders: None,
            generate_next_weeks_needed: false,
            slot_completed_count: 0,
            slot_total_count: 1,
            coming_instructions: vec![],
            completed_instructions: vec![],
            current_instruction: None,
            date: "".to_owned(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn page(
    template: Template,
    user: Option<AuthUser>,
    token: Option<AuthToken>,
    State(app): State<AppState>,
    params: Option<Path<(String,)>>,
    jar: CookieJar,
) -> impl IntoResponse {
    let (Some(user), Some(token)) = (user, token) else {
        return template
            .render(IndexTemplate { show_nav: true })
            .into_response();
    };

    tracing::Span::current().record("user", &user.id);

    let bounds = if let Some(Path((date,))) = params {
        imkitchen_mealplan::month_bounds_from_date(&date, &user.tz)
    } else {
        imkitchen_mealplan::month_bounds_from_now(&user.tz)
    };
    let bounds = crate::try_page_response!(sync: bounds, template);
    let slot = crate::try_page_response!(
        app.mealplan_query.next_slot_from(bounds.date, &user.id),
        template
    );

    let mut slot_completed_count = 0;
    let mut slot_total_count = 1;
    let mut slot_recipe = None;
    let mut completed_instructions = vec![];
    let mut coming_instructions = vec![];
    let mut current_instruction = None;

    if let Some(ref slot) = slot {
        let mut slot_recipe_id = None;
        let mut slot_recipe_status = &slot.main_course.status;

        if let Some(ref appetizer) = slot.appetizer {
            slot_total_count += 1;

            if appetizer.is_completed() {
                slot_completed_count += 1;
            } else if slot_recipe_id.is_none() {
                slot_recipe_id = Some(&appetizer.id);
                slot_recipe_status = &appetizer.status;
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
                slot_recipe_status = &accompaniment.status;
            }
        }

        if let Some(ref dessert) = slot.dessert {
            slot_total_count += 1;

            if dessert.is_completed() {
                slot_completed_count += 1;
            } else if slot_recipe_id.is_none() {
                slot_recipe_id = Some(&dessert.id);
                slot_recipe_status = &dessert.status;
            }
        }

        slot_recipe = crate::try_page_response!(
            app.recipe_query
                .find_user(slot_recipe_id.unwrap_or(&slot.main_course.id)),
            template
        );

        match (slot_recipe_status, slot_recipe.as_ref()) {
            (DaySlotStatus::Idle, Some(recipe)) => {
                coming_instructions = recipe
                    .instructions
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(p, i)| (p, i.description.to_owned()))
                    .collect();

                current_instruction = recipe.instructions.first().map(|i| (0, i.clone()));
            }
            (DaySlotStatus::Cooking(pos), Some(recipe)) => {
                completed_instructions = recipe
                    .instructions
                    .iter()
                    .enumerate()
                    .take(*pos as usize)
                    .map(|(p, i)| (p, i.description.to_owned()))
                    .collect();

                coming_instructions = recipe
                    .instructions
                    .iter()
                    .enumerate()
                    .skip((*pos + 1) as usize)
                    .map(|(p, i)| (p, i.description.to_owned()))
                    .collect();

                current_instruction = recipe.instructions.iter().enumerate().find_map(|(p, i)| {
                    if p == *pos as usize {
                        Some((p, i.clone()))
                    } else {
                        None
                    }
                });
            }
            (DaySlotStatus::Completed, Some(recipe)) => {
                completed_instructions = recipe
                    .instructions
                    .iter()
                    .enumerate()
                    .take(recipe.instructions.len() - 1)
                    .map(|(p, i)| (p, i.description.to_owned()))
                    .collect();
                current_instruction = recipe
                    .instructions
                    .last()
                    .map(|i| (recipe.instructions.len() - 1, i.clone()));
            }
            _ => {}
        };
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

    if let (Some(recipe), Some(slot)) = (slot_recipe.as_mut(), &slot) {
        for ingredient in recipe.ingredients.iter_mut() {
            ingredient.quantity += ((recipe.household_size as u32 * ingredient.quantity
                / slot.household_size as u32) as f64)
                .ceil() as u32;
        }
        recipe.ingredients.sort_by_key(|i| i.name.to_owned());
    }

    let fmt = time::macros::format_description!("[year]-[month]-[day]");
    let date = crate::try_page_response!(sync: bounds.date.format(&fmt), template);

    let auth_cookie = crate::try_page_response!(sync:
        crate::auth::build_cookie(app.config.jwt, token.sub.to_owned(), token.acc.to_owned()),
        template
    );

    let jar = jar.add(auth_cookie);

    (
        jar,
        template.render(KitchenTemplate {
            user,
            slot,
            slot_recipe,
            week,
            prep_remiders,
            generate_next_weeks_needed,
            slot_total_count,
            slot_completed_count,
            completed_instructions,
            coming_instructions,
            current_instruction,
            date,
            ..Default::default()
        }),
    )
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/kitchen-steps.html")]
pub struct KitchenStepsTemplate {
    pub slot_recipe: imkitchen_recipe::query::user::UserView,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub slot: SlotRow,
    pub slot_completed_count: u8,
    pub slot_total_count: u8,
    pub date: String,
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn update_slot_step_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((date, recipe_id, direction)): Path<(String, String, String)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let bounds = crate::try_page_response!(sync: imkitchen_mealplan::month_bounds_from_date(&date, &user.tz), template);
    let mut slot = crate::try_page_response!(opt: app.mealplan_query.next_slot_from(bounds.date, &user.id), template);

    let mut completed_instructions = vec![];
    let mut coming_instructions = vec![];
    let mut slot_completed_count = 0;
    let mut slot_total_count = 1;
    let mut slot_recipe_status = None;

    if slot.main_course.id == recipe_id {
        slot_recipe_status = Some(&slot.main_course.status);
    }

    if let Some(ref appetizer) = slot.appetizer
        && slot_recipe_status.is_none()
        && appetizer.id == recipe_id
    {
        slot_recipe_status = Some(&appetizer.status);
    }

    if let Some(ref accompaniment) = slot.accompaniment
        && slot_recipe_status.is_none()
        && accompaniment.id == recipe_id
    {
        slot_recipe_status = Some(&accompaniment.status);
    }

    if let Some(ref dessert) = slot.dessert
        && slot_recipe_status.is_none()
        && dessert.id == recipe_id
    {
        slot_recipe_status = Some(&dessert.status);
    }

    let Some(slot_recipe_status) = slot_recipe_status else {
        return template.render(NotFoundTemplate);
    };

    let slot_recipe =
        crate::try_page_response!(opt: app.recipe_query.find_user(&recipe_id), template);

    let slot_recipe_status = match (direction.as_str(), slot_recipe_status) {
        ("prev", DaySlotStatus::Idle) => DaySlotStatus::Idle,
        ("prev", DaySlotStatus::Cooking(pos)) => {
            if *pos > 1 {
                DaySlotStatus::Cooking(pos - 1)
            } else {
                DaySlotStatus::Idle
            }
        }
        ("prev", DaySlotStatus::Completed) => {
            DaySlotStatus::Cooking((slot_recipe.instructions.len() - 2) as u8)
        }
        ("next", DaySlotStatus::Idle) => DaySlotStatus::Cooking(1),
        ("next", DaySlotStatus::Cooking(pos)) => {
            println!("{pos}");
            if ((*pos + 1) as usize) < slot_recipe.instructions.len() - 1 {
                DaySlotStatus::Cooking(pos + 1)
            } else {
                DaySlotStatus::Completed
            }
        }
        ("next", DaySlotStatus::Completed) => DaySlotStatus::Completed,
        _ => slot_recipe_status.clone(),
    };

    if slot.main_course.id == recipe_id {
        slot.main_course.status = slot_recipe_status.clone();
    }

    if slot.main_course.is_completed() {
        slot_completed_count += 1;
    }

    if let Some(appetizer) = slot.appetizer.as_mut() {
        slot_total_count += 1;
        if appetizer.id == recipe_id {
            appetizer.status = slot_recipe_status.clone();
        }
        if appetizer.is_completed() {
            slot_completed_count += 1;
        }
    }

    if let Some(accompaniment) = slot.accompaniment.as_mut() {
        slot_total_count += 1;
        if accompaniment.id == recipe_id {
            accompaniment.status = slot_recipe_status.clone();
        }
        if accompaniment.is_completed() {
            slot_completed_count += 1;
        }
    }

    if let Some(dessert) = slot.dessert.as_mut() {
        slot_total_count += 1;
        if dessert.id == recipe_id {
            dessert.status = slot_recipe_status.clone();
        }
        if dessert.is_completed() {
            slot_completed_count += 1;
        }
    }

    let bounds_date = imkitchen_mealplan::date_to_u64(bounds.date);

    crate::try_response!(
        app.mealplan_cmd
            .change_slot_recipe_status(ChangeSlotRecipeStatus {
                user_id: user.id.to_owned(),
                date: bounds_date,
                recipe_id,
                status: slot_recipe_status.clone()
            }),
        template
    );

    let current_instruction = match (&slot_recipe_status, &slot_recipe) {
        (DaySlotStatus::Idle, recipe) => {
            coming_instructions = recipe
                .instructions
                .iter()
                .enumerate()
                .skip(1)
                .map(|(p, i)| (p, i.description.to_owned()))
                .collect();

            recipe.instructions.first().map(|i| (0, i.clone()))
        }
        (DaySlotStatus::Cooking(pos), recipe) => {
            completed_instructions = recipe
                .instructions
                .iter()
                .enumerate()
                .take(*pos as usize)
                .map(|(p, i)| (p, i.description.to_owned()))
                .collect();

            coming_instructions = recipe
                .instructions
                .iter()
                .enumerate()
                .skip((*pos + 1) as usize)
                .map(|(p, i)| (p, i.description.to_owned()))
                .collect();

            recipe.instructions.iter().enumerate().find_map(|(p, i)| {
                if p == *pos as usize {
                    Some((p, i.clone()))
                } else {
                    None
                }
            })
        }
        (DaySlotStatus::Completed, recipe) => {
            completed_instructions = recipe
                .instructions
                .iter()
                .enumerate()
                .take(recipe.instructions.len() - 1)
                .map(|(p, i)| (p, i.description.to_owned()))
                .collect();
            recipe
                .instructions
                .last()
                .map(|i| (recipe.instructions.len() - 1, i.clone()))
        }
    };

    template
        .render(KitchenStepsTemplate {
            slot,
            slot_recipe,
            slot_total_count,
            slot_completed_count,
            completed_instructions,
            coming_instructions,
            current_instruction,
            date,
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/kitchen-dish.html")]
pub struct KitchenDishTemplate {
    pub date: String,
    pub slot_recipe: imkitchen_recipe::query::user::UserView,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn select_dish(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((date, recipe_id)): Path<(String, String)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let bounds = crate::try_page_response!(sync: imkitchen_mealplan::month_bounds_from_date(&date, &user.tz), template);
    let slot = crate::try_page_response!(opt: app.mealplan_query.next_slot_from(bounds.date, &user.id), template);

    let mut completed_instructions = vec![];
    let mut coming_instructions = vec![];
    let mut slot_recipe_status = None;

    if slot.main_course.id == recipe_id {
        slot_recipe_status = Some(&slot.main_course.status);
    }

    if let Some(ref appetizer) = slot.appetizer
        && slot_recipe_status.is_none()
        && appetizer.id == recipe_id
    {
        slot_recipe_status = Some(&appetizer.status);
    }

    if let Some(ref accompaniment) = slot.accompaniment
        && slot_recipe_status.is_none()
        && accompaniment.id == recipe_id
    {
        slot_recipe_status = Some(&accompaniment.status);
    }

    if let Some(ref dessert) = slot.dessert
        && slot_recipe_status.is_none()
        && dessert.id == recipe_id
    {
        slot_recipe_status = Some(&dessert.status);
    }

    let Some(slot_recipe_status) = slot_recipe_status else {
        return template.render(NotFoundTemplate);
    };

    let mut slot_recipe =
        crate::try_page_response!(opt: app.recipe_query.find_user(&recipe_id), template);

    for ingredient in slot_recipe.ingredients.iter_mut() {
        ingredient.quantity += ((slot_recipe.household_size as u32 * ingredient.quantity
            / slot.household_size as u32) as f64)
            .ceil() as u32;
    }

    slot_recipe.ingredients.sort_by_key(|i| i.name.to_owned());

    let current_instruction = match (&slot_recipe_status, &slot_recipe) {
        (DaySlotStatus::Idle, recipe) => {
            coming_instructions = recipe
                .instructions
                .iter()
                .enumerate()
                .skip(1)
                .map(|(p, i)| (p, i.description.to_owned()))
                .collect();

            recipe.instructions.first().map(|i| (0, i.clone()))
        }
        (DaySlotStatus::Cooking(pos), recipe) => {
            completed_instructions = recipe
                .instructions
                .iter()
                .enumerate()
                .take(*pos as usize)
                .map(|(p, i)| (p, i.description.to_owned()))
                .collect();

            coming_instructions = recipe
                .instructions
                .iter()
                .enumerate()
                .skip((*pos + 1) as usize)
                .map(|(p, i)| (p, i.description.to_owned()))
                .collect();

            recipe.instructions.iter().enumerate().find_map(|(p, i)| {
                if p == *pos as usize {
                    Some((p, i.clone()))
                } else {
                    None
                }
            })
        }
        (DaySlotStatus::Completed, recipe) => {
            completed_instructions = recipe
                .instructions
                .iter()
                .enumerate()
                .take(recipe.instructions.len() - 1)
                .map(|(p, i)| (p, i.description.to_owned()))
                .collect();
            recipe
                .instructions
                .last()
                .map(|i| (recipe.instructions.len() - 1, i.clone()))
        }
    };

    template
        .render(KitchenDishTemplate {
            slot_recipe,
            completed_instructions,
            coming_instructions,
            current_instruction,
            date,
        })
        .into_response()
}
