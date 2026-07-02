use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::CookieJar;
use imkitchen_core::mealplan::slot::SlotRow;
use imkitchen_core::mealplan::{ChangeSlotRecipeStatus, Recipe};
use imkitchen_types::mealplan::DaySlotStatus;
use imkitchen_types::recipe::{Instruction, IngredientUnitFormat};
use imkitchen_types::{mealplan::DaySlotRecipe, recipe::RecipeType};

pub use imkitchen_web_shared::config;

use imkitchen_web_shared::AppState;
use imkitchen_web_shared::auth::{AuthToken, AuthUser, RequirePremium};
use imkitchen_web_shared::template::{NotFoundTemplate, Template, filters};

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub show_nav: bool,
}

#[derive(askama::Template)]
#[template(path = "onboarding-recipe.html")]
pub struct OnboardingRecipeTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub today_unix: u64,
}

#[derive(askama::Template)]
#[template(path = "onboarding-menu.html")]
pub struct OnboardingMenuTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub recipes: Vec<Recipe>,
    pub main_count: usize,
    pub appetizer_count: usize,
    pub accompaniment_count: usize,
    pub dessert_count: usize,
}

#[derive(Default, Clone)]
pub struct KitchenWeekDay {
    pub date: String,
    pub day_num: u8,
    pub weekday: String,
    pub is_today: bool,
    pub meal_types: Vec<RecipeType>,
}

#[derive(askama::Template)]
#[template(path = "kitchen.html")]
pub struct KitchenTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub slot: Option<SlotRow>,
    pub slot_recipe: Option<imkitchen_core::recipe::query::user::UserView>,
    pub slot_completed_count: u8,
    pub slot_total_count: u8,
    pub prep_remiders: Option<Vec<DaySlotRecipe>>,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub date: String,
    pub week_days: Vec<KitchenWeekDay>,
    /// Recipe id → slug for the prep-reminder cards, so they can link to the
    /// canonical `/r/{slug}` detail page. Missing ids fall back to the id.
    pub slugs: std::collections::HashMap<String, String>,
    /// When true, the "Start cooking" button links to the recipe's original URL
    /// (external) instead of the in-app cooking screen. See [`cook_is_external`].
    pub cook_external: bool,
}

impl KitchenTemplate {
    /// Slug for a slot recipe id, falling back to the id when unknown (the
    /// `/r/{param}` route also accepts a raw id).
    pub fn dish_slug<'a>(&'a self, id: &'a str) -> &'a str {
        self.slugs.get(id).map(String::as_str).unwrap_or(id)
    }
}

impl Default for KitchenTemplate {
    fn default() -> Self {
        Self {
            current_path: "kitchen".to_owned(),
            user: AuthUser::default(),
            slot: None,
            slot_recipe: None,
            prep_remiders: None,
            slot_completed_count: 0,
            slot_total_count: 1,
            coming_instructions: vec![],
            completed_instructions: vec![],
            current_instruction: None,
            date: "".to_owned(),
            week_days: vec![],
            slugs: std::collections::HashMap::new(),
            cook_external: false,
        }
    }
}

/// Whether the "Start cooking" button should link straight to the recipe's
/// original URL (opened externally) rather than into the in-app cooking screen.
///
/// True only when the recipe has no parsed steps AND its origin refuses framing —
/// i.e. there is nothing to cook in-app. The embeddability check runs only for
/// step-less recipes (and is cached per-domain), so recipes with steps never
/// trigger a network call here.
async fn cook_is_external(
    app: &AppState,
    slot_recipe: &imkitchen_core::recipe::query::user::UserView,
    current_instruction: &Option<(usize, Instruction)>,
) -> bool {
    if current_instruction.is_some() {
        return false;
    }
    match slot_recipe.origin.as_deref() {
        Some(origin) => !app
            .core
            .recipe
            .is_origin_embeddable(origin)
            .await
            .unwrap_or(false),
        None => false,
    }
}

/// A single grocery aisle section of a recipe's ingredient list, keyed by the
/// `shopping_<Category>` string so the cooking-screen template can reuse the
/// groceries aisle macros (emoji / label / accent color).
pub struct IngredientAisle {
    pub name: String,
    pub items: Vec<imkitchen_types::recipe::Ingredient>,
}

/// Scale a recipe's ingredient quantities to the meal-plan slot's household size
/// and sort them by name. Shared by every kitchen screen that shows ingredients
/// (dashboard, dish preview, and the cooking screen) so they stay consistent.
fn scale_ingredients(
    recipe: &mut imkitchen_core::recipe::query::user::UserView,
    slot_household_size: u16,
) {
    // Recipes are authored for `recipe.household_size` servings, which also acts
    // as the recipe's minimum: a recipe can't realistically be made for fewer
    // servings than it was written for (e.g. a whole chicken serves 4). So scale
    // to `max(recipe, slot)` — up for larger households, never below the recipe's
    // own size. Guard the divisor since household size is an unvalidated field.
    let recipe_household_size = recipe.household_size.max(1);
    let serving_target = recipe_household_size.max(slot_household_size);
    for ingredient in recipe.ingredients.iter_mut() {
        ingredient.quantity = (ingredient.quantity as f64 * serving_target as f64
            / recipe_household_size as f64)
            .ceil() as u32;
    }
    recipe.ingredients.sort_by_key(|i| i.name.to_owned());
}

/// Group (already-scaled) ingredients into ordered aisle sections keyed by
/// `shopping_<Category>`, mirroring the groceries page grouping.
fn group_ingredients_by_aisle(
    ingredients: &[imkitchen_types::recipe::Ingredient],
) -> Vec<IngredientAisle> {
    let mut categories: std::collections::HashMap<
        String,
        Vec<imkitchen_types::recipe::Ingredient>,
    > = std::collections::HashMap::new();

    for ingredient in ingredients.iter() {
        let key = match &ingredient.category {
            Some(c) => format!("shopping_{c}"),
            None => "shopping_Unknown".to_owned(),
        };
        categories.entry(key).or_default().push(ingredient.clone());
    }

    let mut aisles = categories
        .into_iter()
        .map(|(name, items)| IngredientAisle { name, items })
        .collect::<Vec<_>>();

    aisles.sort_by(|a, b| a.name.cmp(&b.name));

    aisles
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn kitchen_page(
    template: Template,
    user: AuthUser,
    token: Option<AuthToken>,
    state: State<AppState>,
    params: Option<Path<(String,)>>,
    jar: CookieJar,
) -> impl IntoResponse {
    page(template, Some(user), token, state, params, jar).await
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
        imkitchen_core::mealplan::month_bounds_from_date(&date, &user.tz)
    } else {
        imkitchen_core::mealplan::month_bounds_from_now(&user.tz)
    };
    let bounds = imkitchen_web_shared::try_page_response!(sync: bounds, template);
    let slot = imkitchen_web_shared::try_page_response!(
        app.core.mealplan.next_slot_from(bounds.date, &user.id),
        template
    );

    if slot.is_none() {
        let main_courses = imkitchen_web_shared::try_page_response!(
            app.core
                .mealplan
                .first_week_recipes(&user.id, RecipeType::MainCourse),
            template
        );

        if main_courses.is_empty() {
            let today_unix = imkitchen_core::mealplan::now(&user.tz).unix_timestamp() as u64;
            return template
                .render(OnboardingRecipeTemplate {
                    current_path: "kitchen".to_owned(),
                    user,
                    today_unix,
                })
                .into_response();
        }

        let appetizers = imkitchen_web_shared::try_page_response!(
            app.core
                .mealplan
                .first_week_recipes(&user.id, RecipeType::Appetizer),
            template
        );
        let accompaniments = imkitchen_web_shared::try_page_response!(
            app.core
                .mealplan
                .first_week_recipes(&user.id, RecipeType::Accompaniment),
            template
        );
        let desserts = imkitchen_web_shared::try_page_response!(
            app.core
                .mealplan
                .first_week_recipes(&user.id, RecipeType::Dessert),
            template
        );

        return template
            .render(OnboardingMenuTemplate {
                current_path: "kitchen".to_owned(),
                user,
                main_count: main_courses.len(),
                appetizer_count: appetizers.len(),
                accompaniment_count: accompaniments.len(),
                dessert_count: desserts.len(),
                recipes: main_courses,
            })
            .into_response();
    }

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

        if let Some(ref beverage) = slot.beverage {
            slot_total_count += 1;

            if beverage.is_completed() {
                slot_completed_count += 1;
            } else if slot_recipe_id.is_none() {
                slot_recipe_id = Some(&beverage.id);
                slot_recipe_status = &beverage.status;
            }
        }

        if let Some(ref condiment) = slot.condiment {
            slot_total_count += 1;

            if condiment.is_completed() {
                slot_completed_count += 1;
            } else if slot_recipe_id.is_none() {
                slot_recipe_id = Some(&condiment.id);
                slot_recipe_status = &condiment.status;
            }
        }

        slot_recipe = imkitchen_web_shared::try_page_response!(
            app.core
                .recipe
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
        imkitchen_web_shared::try_page_response!(
            app.core
                .mealplan
                .next_prep_remiders_from(slot.day, &user.id),
            template
        )
    } else {
        None
    };

    let slugs = if let Some(ref remiders) = prep_remiders {
        imkitchen_web_shared::try_page_response!(
            app.core
                .recipe
                .slugs(remiders.iter().map(|r| r.id.to_owned()).collect()),
            template
        )
    } else {
        std::collections::HashMap::new()
    };

    if let (Some(recipe), Some(slot)) = (slot_recipe.as_mut(), &slot) {
        scale_ingredients(recipe, slot.household_size);
    }

    let fmt = time::macros::format_description!("[year]-[month]-[day]");
    let date = imkitchen_web_shared::try_page_response!(sync: bounds.date.format(&fmt), template);

    // ── Week strip — 7 days Mon-Sun anchored on today (user tz) ──────
    let today = imkitchen_core::mealplan::now(&user.tz);
    let today_u64 = imkitchen_core::mealplan::date_to_u64(today);
    let mut week_dates: Vec<time::OffsetDateTime> =
        imkitchen_core::mealplan::week_days_before(today);
    week_dates.push(today);
    week_dates.extend(imkitchen_core::mealplan::week_days_after(today));

    let (week_start, week_end) = (
        week_dates.first().copied().unwrap_or(today),
        week_dates.last().copied().unwrap_or(today),
    );

    let week_slots = imkitchen_web_shared::try_page_response!(
        app.core.mealplan.range(&user.id, week_start, week_end),
        template
    );

    let week_days: Vec<KitchenWeekDay> = week_dates
        .iter()
        .map(|d| {
            let d_u64 = imkitchen_core::mealplan::date_to_u64(*d);
            let matching = week_slots.iter().find(|s| {
                time::OffsetDateTime::from_unix_timestamp(s.day as i64)
                    .map(|sd| imkitchen_core::mealplan::date_to_u64(sd) == d_u64)
                    .unwrap_or(false)
            });
            let mut meal_types = vec![];
            if let Some(s) = matching {
                if s.appetizer.is_some() {
                    meal_types.push(RecipeType::Appetizer);
                }
                meal_types.push(RecipeType::MainCourse);
                if s.accompaniment.is_some() {
                    meal_types.push(RecipeType::Accompaniment);
                }
                if s.dessert.is_some() {
                    meal_types.push(RecipeType::Dessert);
                }
                if s.beverage.is_some() {
                    meal_types.push(RecipeType::Beverage);
                }
                if s.condiment.is_some() {
                    meal_types.push(RecipeType::Condiment);
                }
            }
            KitchenWeekDay {
                date: d.format(&fmt).unwrap_or_default(),
                day_num: d.day(),
                weekday: d.weekday().to_string().chars().take(3).collect(),
                is_today: d_u64 == today_u64,
                meal_types,
            }
        })
        .collect();

    let cook_external = match slot_recipe.as_ref() {
        Some(recipe) => cook_is_external(&app, recipe, &current_instruction).await,
        None => false,
    };

    let auth_cookie = imkitchen_web_shared::try_page_response!(sync:
        imkitchen_web_shared::auth::build_cookie(app.config.jwt, token.sub.to_owned(), token.acc.to_owned()),
        template
    );

    let jar = jar.add(auth_cookie);

    (
        jar,
        template.render(KitchenTemplate {
            user,
            slot,
            slot_recipe,
            prep_remiders,
            slot_total_count,
            slot_completed_count,
            completed_instructions,
            coming_instructions,
            current_instruction,
            date,
            week_days,
            slugs,
            cook_external,
            ..Default::default()
        }),
    )
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "cooking.html")]
pub struct CookingTemplate {
    pub slot_recipe: imkitchen_core::recipe::query::user::UserView,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub date: String,
    pub show_iframe: bool,
    /// When true, render the ingredient list (grouped in `ingredient_aisles`)
    /// as the first screen of the cooking flow instead of a step.
    pub show_ingredients: bool,
    pub ingredient_aisles: Vec<IngredientAisle>,
}

// Fragment version of CookingTemplate — same fields, but renders only the
// #cooking-screen partial so it can be swapped in place by TwinSpark.
#[derive(askama::Template)]
#[template(path = "partials/cooking-screen.html")]
pub struct CookingScreenTemplate {
    pub slot_recipe: imkitchen_core::recipe::query::user::UserView,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub date: String,
    pub show_iframe: bool,
    pub show_ingredients: bool,
    pub ingredient_aisles: Vec<IngredientAisle>,
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn update_slot_step_action(
    template: Template,
    RequirePremium(user): RequirePremium,
    State(app): State<AppState>,
    Path((date, recipe_id, direction)): Path<(String, String, String)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let bounds = imkitchen_web_shared::try_page_response!(sync: imkitchen_core::mealplan::month_bounds_from_date(&date, &user.tz), template);
    let slot = imkitchen_web_shared::try_page_response!(opt: app.core.mealplan.next_slot_from(bounds.date, &user.id), template);

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

    if let Some(ref beverage) = slot.beverage
        && slot_recipe_status.is_none()
        && beverage.id == recipe_id
    {
        slot_recipe_status = Some(&beverage.status);
    }

    if let Some(ref condiment) = slot.condiment
        && slot_recipe_status.is_none()
        && condiment.id == recipe_id
    {
        slot_recipe_status = Some(&condiment.status);
    }

    let Some(slot_recipe_status) = slot_recipe_status else {
        return template.render(NotFoundTemplate).into_response();
    };

    let mut slot_recipe = imkitchen_web_shared::try_page_response!(opt: app.core.recipe.find_user(&recipe_id), template);
    scale_ingredients(&mut slot_recipe, slot.household_size);

    // `Idle` is the ingredients screen; `Cooking(0)` is the first instruction,
    // `Cooking(len-2)` the second-to-last, and `Completed` the last.
    let len = slot_recipe.instructions.len();
    let slot_recipe_status = match (direction.as_str(), slot_recipe_status) {
        ("prev", DaySlotStatus::Idle) => DaySlotStatus::Idle,
        ("prev", DaySlotStatus::Cooking(pos)) => {
            if *pos == 0 {
                DaySlotStatus::Idle
            } else {
                DaySlotStatus::Cooking(pos - 1)
            }
        }
        ("prev", DaySlotStatus::Completed) => {
            if len <= 1 {
                DaySlotStatus::Idle
            } else {
                DaySlotStatus::Cooking((len - 2) as u8)
            }
        }
        ("next", DaySlotStatus::Idle) => {
            if len <= 1 {
                DaySlotStatus::Completed
            } else {
                DaySlotStatus::Cooking(0)
            }
        }
        ("next", DaySlotStatus::Cooking(pos)) => {
            if ((*pos + 1) as usize) < len - 1 {
                DaySlotStatus::Cooking(pos + 1)
            } else {
                DaySlotStatus::Completed
            }
        }
        ("next", DaySlotStatus::Completed) => DaySlotStatus::Completed,
        _ => slot_recipe_status.clone(),
    };

    let bounds_date = imkitchen_core::mealplan::date_to_u64(bounds.date);

    imkitchen_web_shared::try_response!(
        app.core
            .mealplan
            .change_slot_recipe_status(ChangeSlotRecipeStatus {
                user_id: user.id.to_owned(),
                date: bounds_date,
                recipe_id: recipe_id.clone(),
                status: slot_recipe_status.clone()
            }),
        template
    );

    // Compute view state from the NEW status in-memory — re-reading the projection
    // here would race with evento's async projection update and show stale state
    // (each Next would appear to leave the user on the same step).
    let mut completed_instructions = vec![];
    let mut coming_instructions = vec![];
    let current_instruction = match (&slot_recipe_status, &slot_recipe) {
        // Ingredients screen — no current step.
        (DaySlotStatus::Idle, _) => None,
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

    // Ingredient list is the first screen of the cooking flow — shown while the
    // recipe is Idle, but only when it actually has in-app steps to cook.
    let show_ingredients =
        matches!(slot_recipe_status, DaySlotStatus::Idle) && !slot_recipe.instructions.is_empty();
    let ingredient_aisles = if show_ingredients {
        group_ingredients_by_aisle(&slot_recipe.ingredients)
    } else {
        vec![]
    };

    let show_iframe = match slot_recipe.origin.as_deref() {
        Some(origin) => app
            .core
            .recipe
            .is_origin_embeddable(origin)
            .await
            .unwrap_or(false),
        None => false,
    };

    template
        .render(CookingScreenTemplate {
            slot_recipe,
            completed_instructions,
            coming_instructions,
            current_instruction,
            date,
            show_iframe,
            show_ingredients,
            ingredient_aisles,
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/kitchen-dish.html")]
pub struct KitchenDishTemplate {
    pub date: String,
    pub slot: SlotRow,
    pub slot_recipe: imkitchen_core::recipe::query::user::UserView,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub cook_external: bool,
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn select_dish(
    template: Template,
    RequirePremium(user): RequirePremium,
    State(app): State<AppState>,
    Path((date, recipe_id)): Path<(String, String)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let bounds = imkitchen_web_shared::try_page_response!(sync: imkitchen_core::mealplan::month_bounds_from_date(&date, &user.tz), template);
    let slot = imkitchen_web_shared::try_page_response!(opt: app.core.mealplan.next_slot_from(bounds.date, &user.id), template);

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

    if let Some(ref beverage) = slot.beverage
        && slot_recipe_status.is_none()
        && beverage.id == recipe_id
    {
        slot_recipe_status = Some(&beverage.status);
    }

    if let Some(ref condiment) = slot.condiment
        && slot_recipe_status.is_none()
        && condiment.id == recipe_id
    {
        slot_recipe_status = Some(&condiment.status);
    }

    let Some(slot_recipe_status) = slot_recipe_status else {
        return template.render(NotFoundTemplate);
    };

    let mut slot_recipe = imkitchen_web_shared::try_page_response!(opt: app.core.recipe.find_user(&recipe_id), template);

    scale_ingredients(&mut slot_recipe, slot.household_size);

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

    let cook_external = cook_is_external(&app, &slot_recipe, &current_instruction).await;

    template
        .render(KitchenDishTemplate {
            slot,
            slot_recipe,
            completed_instructions,
            coming_instructions,
            current_instruction,
            date,
            cook_external,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn cook_page(
    template: Template,
    RequirePremium(user): RequirePremium,
    State(app): State<AppState>,
    Path((date, recipe_id)): Path<(String, String)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let bounds = imkitchen_web_shared::try_page_response!(sync: imkitchen_core::mealplan::month_bounds_from_date(&date, &user.tz), template);
    let slot = imkitchen_web_shared::try_page_response!(opt: app.core.mealplan.next_slot_from(bounds.date, &user.id), template);

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
    if let Some(ref beverage) = slot.beverage
        && slot_recipe_status.is_none()
        && beverage.id == recipe_id
    {
        slot_recipe_status = Some(&beverage.status);
    }
    if let Some(ref condiment) = slot.condiment
        && slot_recipe_status.is_none()
        && condiment.id == recipe_id
    {
        slot_recipe_status = Some(&condiment.status);
    }

    let Some(slot_recipe_status) = slot_recipe_status else {
        return template.render(NotFoundTemplate).into_response();
    };

    let mut slot_recipe = imkitchen_web_shared::try_page_response!(opt: app.core.recipe.find_user(&recipe_id), template);
    scale_ingredients(&mut slot_recipe, slot.household_size);

    // `Idle` renders the ingredient list (first screen); `Cooking(0)` is the
    // first instruction and `Completed` the last.
    let current_instruction = match (&slot_recipe_status, &slot_recipe) {
        (DaySlotStatus::Idle, _) => None,
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

    // Ingredient list is the first screen — shown while Idle, but only when the
    // recipe actually has in-app steps to cook.
    let show_ingredients =
        matches!(slot_recipe_status, DaySlotStatus::Idle) && !slot_recipe.instructions.is_empty();
    let ingredient_aisles = if show_ingredients {
        group_ingredients_by_aisle(&slot_recipe.ingredients)
    } else {
        vec![]
    };

    let show_iframe = match slot_recipe.origin.as_deref() {
        Some(origin) => app
            .core
            .recipe
            .is_origin_embeddable(origin)
            .await
            .unwrap_or(false),
        None => false,
    };

    // Imported recipe with no parsed steps whose origin refuses framing: there is
    // nothing to show in-app, so send the user straight to the original instead of
    // rendering a "Open Original Recipe" button they'd have to tap.
    if !show_iframe
        && slot_recipe.instructions.is_empty()
        && let Some(origin) = slot_recipe.origin.as_deref()
    {
        return Redirect::to(origin).into_response();
    }

    template
        .render(CookingTemplate {
            slot_recipe,
            completed_instructions,
            coming_instructions,
            current_instruction,
            date,
            show_iframe,
            show_ingredients,
            ingredient_aisles,
        })
        .into_response()
}

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/", get(page))
        .route(
            "/kitchen/{date}/{recipe_id}/step/{direction}",
            post(update_slot_step_action),
        )
        .route("/kitchen/{date}/{recipe_id}/select-dish", post(select_dish))
        .route("/kitchen/{date}/{recipe_id}/cook", get(cook_page))
        .route("/kitchen/{date}", get(kitchen_page))
}
