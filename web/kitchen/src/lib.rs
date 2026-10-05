//! The kitchen: what to cook next from the user's recipe list, and the
//! step-by-step cooking screens.

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::CookieJar;
use imkitchen_core::recipe::query::user::{RecipeCard, UserView};
use imkitchen_core::shopping::{ChangeRecipeStatus, PoolRecipe, ShoppingState};
use imkitchen_types::recipe::{IngredientUnitFormat, Instruction, RecipeType};
use imkitchen_types::shopping::RecipeStatus;

pub use imkitchen_web_shared::config;

use imkitchen_web_shared::AppState;
use imkitchen_web_shared::auth::{AuthToken, AuthUser};
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
}

#[derive(askama::Template)]
#[template(path = "onboarding-menu.html")]
pub struct OnboardingMenuTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub recipes: Vec<PoolRecipe>,
    pub main_count: usize,
    pub appetizer_count: usize,
    pub accompaniment_count: usize,
    pub dessert_count: usize,
}

/// One recipe of the list as the kitchen shows it: course, cooking status and
/// the advance-prep note (for the "Prep ahead" rail).
#[derive(Clone, Debug)]
pub struct ListEntry {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub recipe_type: RecipeType,
    pub status: RecipeStatus,
    pub advance_prep: String,
    pub prep_time: u16,
    pub cook_time: u16,
}

impl ListEntry {
    pub fn total_time(&self) -> u16 {
        self.prep_time + self.cook_time
    }

    pub fn is_completed(&self) -> bool {
        self.status.is_completed()
    }

    pub fn is_cooking(&self) -> bool {
        self.status.is_cooking()
    }
}

#[derive(askama::Template)]
#[template(path = "kitchen.html")]
pub struct KitchenTemplate {
    pub current_path: String,
    pub user: AuthUser,
    /// Every recipe in the list, courses first (starter → main → side →
    /// dessert → drink → sauce), then list order.
    pub entries: Vec<ListEntry>,
    /// The recipe to cook next: the first one not yet cooked.
    pub focused: Option<UserView>,
    pub focused_status: RecipeStatus,
    pub completed_count: usize,
    pub total_count: usize,
    /// Uncooked recipes (other than the focused one) with an advance-prep note.
    pub prep_ahead: Vec<ListEntry>,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    /// When true, the "Start cooking" button links to the recipe's original URL
    /// (external) instead of the in-app cooking screen. See [`cook_is_external`].
    pub cook_external: bool,
}

impl Default for KitchenTemplate {
    fn default() -> Self {
        Self {
            current_path: "kitchen".to_owned(),
            user: AuthUser::default(),
            entries: vec![],
            focused: None,
            focused_status: RecipeStatus::Idle,
            completed_count: 0,
            total_count: 0,
            prep_ahead: vec![],
            coming_instructions: vec![],
            completed_instructions: vec![],
            current_instruction: None,
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
    recipe: &UserView,
    current_instruction: &Option<(usize, Instruction)>,
) -> bool {
    if current_instruction.is_some() {
        return false;
    }
    match recipe.origin.as_deref() {
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

/// Scale a recipe's ingredient quantities to the user's household size and
/// sort them by name. Shared by every kitchen screen that shows ingredients
/// (dashboard, dish preview, and the cooking screen) so they stay consistent.
fn scale_ingredients(recipe: &mut UserView, household_size: u16) {
    // Recipes are authored for `recipe.household_size` servings, which also acts
    // as the recipe's minimum: a recipe can't realistically be made for fewer
    // servings than it was written for (e.g. a whole chicken serves 4). So scale
    // to `max(recipe, household)` — up for larger households, never below the
    // recipe's own size. Guard the divisor since household size is an
    // unvalidated field.
    let recipe_household_size = recipe.household_size.max(1);
    let serving_target = recipe_household_size.max(household_size);
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

/// Course order on the kitchen page: eat a starter before the main, etc.
fn course_rank(recipe_type: &RecipeType) -> u8 {
    match recipe_type {
        RecipeType::Appetizer => 0,
        RecipeType::MainCourse => 1,
        RecipeType::Accompaniment => 2,
        RecipeType::Dessert => 3,
        RecipeType::Beverage => 4,
        RecipeType::Condiment => 5,
    }
}

/// The list as kitchen entries: courses first, then list order. Ids whose
/// recipe no longer exists are skipped.
pub fn list_entries(state: &ShoppingState, cards: Vec<RecipeCard>) -> Vec<ListEntry> {
    let position = |id: &str| state.recipe_ids.iter().position(|x| x == id);
    let mut entries: Vec<(usize, ListEntry)> = cards
        .into_iter()
        .filter_map(|card| {
            let pos = position(&card.id)?;
            Some((
                pos,
                ListEntry {
                    status: state.status(&card.id),
                    id: card.id,
                    name: card.name,
                    slug: card.slug,
                    recipe_type: card.recipe_type.0,
                    advance_prep: card.advance_prep,
                    prep_time: card.prep_time,
                    cook_time: card.cook_time,
                },
            ))
        })
        .collect();
    entries.sort_by_key(|(pos, e)| (course_rank(&e.recipe_type), *pos));
    entries.into_iter().map(|(_, e)| e).collect()
}

/// `(completed, coming, current)` instructions around the cooking cursor.
type StepView = (
    Vec<(usize, String)>,
    Vec<(usize, String)>,
    Option<(usize, Instruction)>,
);

/// Split a recipe's instructions around the cooking cursor.
///
/// `Idle` means the recipe has not been started: the dashboard previews the
/// first step (`idle_shows_first_step`), while the cooking screen shows the
/// ingredient list instead and has no current step.
fn split_instructions(
    recipe: &UserView,
    status: &RecipeStatus,
    idle_shows_first_step: bool,
) -> StepView {
    let describe = |(p, i): (usize, &Instruction)| (p, i.description.to_owned());
    match status {
        RecipeStatus::Idle if idle_shows_first_step => (
            vec![],
            recipe
                .instructions
                .iter()
                .enumerate()
                .skip(1)
                .map(describe)
                .collect(),
            recipe.instructions.first().map(|i| (0, i.clone())),
        ),
        RecipeStatus::Idle => (vec![], vec![], None),
        RecipeStatus::Cooking(pos) => {
            let pos = *pos as usize;
            (
                recipe
                    .instructions
                    .iter()
                    .enumerate()
                    .take(pos)
                    .map(describe)
                    .collect(),
                recipe
                    .instructions
                    .iter()
                    .enumerate()
                    .skip(pos + 1)
                    .map(describe)
                    .collect(),
                recipe.instructions.get(pos).map(|i| (pos, i.clone())),
            )
        }
        RecipeStatus::Completed => {
            let len = recipe.instructions.len();
            (
                recipe
                    .instructions
                    .iter()
                    .enumerate()
                    .take(len.saturating_sub(1))
                    .map(describe)
                    .collect(),
                vec![],
                recipe
                    .instructions
                    .last()
                    .map(|i| (len.saturating_sub(1), i.clone())),
            )
        }
    }
}

/// Move the cooking cursor one step. `Idle` is the ingredients screen;
/// `Cooking(0)` is the first instruction, `Cooking(len-2)` the second-to-last,
/// and `Completed` the last.
fn next_status(direction: &str, current: &RecipeStatus, len: usize) -> RecipeStatus {
    match (direction, current) {
        ("prev", RecipeStatus::Idle) => RecipeStatus::Idle,
        ("prev", RecipeStatus::Cooking(pos)) => {
            if *pos == 0 {
                RecipeStatus::Idle
            } else {
                RecipeStatus::Cooking(pos - 1)
            }
        }
        ("prev", RecipeStatus::Completed) => {
            if len <= 1 {
                RecipeStatus::Idle
            } else {
                RecipeStatus::Cooking((len - 2) as u8)
            }
        }
        ("next", RecipeStatus::Idle) => {
            if len <= 1 {
                RecipeStatus::Completed
            } else {
                RecipeStatus::Cooking(0)
            }
        }
        ("next", RecipeStatus::Cooking(pos)) => {
            if ((*pos + 1) as usize) < len - 1 {
                RecipeStatus::Cooking(pos + 1)
            } else {
                RecipeStatus::Completed
            }
        }
        ("next", RecipeStatus::Completed) => RecipeStatus::Completed,
        _ => current.clone(),
    }
}

/// The list state plus the household size it was scaled for.
struct ListContext {
    state: ShoppingState,
    household_size: u16,
}

async fn load_list(app: &AppState, user_id: &str) -> anyhow::Result<ListContext> {
    let household_size = app
        .identity
        .meal_preferences
        .load(user_id)
        .await?
        .household_size;
    let state = app.core.shopping.state(user_id, household_size).await?;
    Ok(ListContext {
        state,
        household_size,
    })
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

    let list = imkitchen_web_shared::try_page_response!(load_list(&app, &user.id), template);

    if list.state.recipe_ids.is_empty() {
        let main_courses = imkitchen_web_shared::try_page_response!(
            app.core
                .shopping
                .sample_recipes(&user.id, RecipeType::MainCourse),
            template
        );

        if main_courses.is_empty() {
            return template
                .render(OnboardingRecipeTemplate {
                    current_path: "kitchen".to_owned(),
                    user,
                })
                .into_response();
        }

        let appetizers = imkitchen_web_shared::try_page_response!(
            app.core
                .shopping
                .sample_recipes(&user.id, RecipeType::Appetizer),
            template
        );
        let accompaniments = imkitchen_web_shared::try_page_response!(
            app.core
                .shopping
                .sample_recipes(&user.id, RecipeType::Accompaniment),
            template
        );
        let desserts = imkitchen_web_shared::try_page_response!(
            app.core
                .shopping
                .sample_recipes(&user.id, RecipeType::Dessert),
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

    let cards = imkitchen_web_shared::try_page_response!(
        app.core.recipe.filter_by_ids(list.state.recipe_ids.clone()),
        template
    );
    let entries = list_entries(&list.state, cards);

    let total_count = entries.len();
    let completed_count = entries.iter().filter(|e| e.is_completed()).count();

    let focused_entry = entries
        .iter()
        .find(|e| !e.is_completed())
        .or_else(|| entries.first())
        .cloned();

    let mut focused = None;
    let mut focused_status = RecipeStatus::Idle;
    let mut completed_instructions = vec![];
    let mut coming_instructions = vec![];
    let mut current_instruction = None;

    if let Some(entry) = &focused_entry {
        focused = imkitchen_web_shared::try_page_response!(
            app.core.recipe.find_user(&entry.id),
            template
        );
        focused_status = entry.status.clone();
        if let Some(recipe) = focused.as_mut() {
            scale_ingredients(recipe, list.household_size);
            (
                completed_instructions,
                coming_instructions,
                current_instruction,
            ) = split_instructions(recipe, &focused_status, true);
        }
    }

    let prep_ahead: Vec<ListEntry> = entries
        .iter()
        .filter(|e| !e.advance_prep.trim().is_empty() && !e.is_completed())
        .filter(|e| focused_entry.as_ref().is_none_or(|f| f.id != e.id))
        .cloned()
        .collect();

    let cook_external = match focused.as_ref() {
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
            entries,
            focused,
            focused_status,
            completed_count,
            total_count,
            prep_ahead,
            completed_instructions,
            coming_instructions,
            current_instruction,
            cook_external,
            ..Default::default()
        }),
    )
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "cooking.html")]
pub struct CookingTemplate {
    pub slot_recipe: UserView,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
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
    pub slot_recipe: UserView,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub show_iframe: bool,
    pub show_ingredients: bool,
    pub ingredient_aisles: Vec<IngredientAisle>,
}

/// A recipe from the list, scaled, with its cooking status. `NotFound` when
/// the recipe is not in the list.
async fn find_list_recipe(
    app: &AppState,
    list: &ListContext,
    recipe_id: &str,
) -> anyhow::Result<Option<(UserView, RecipeStatus)>> {
    if !list.state.contains(recipe_id) {
        return Ok(None);
    }
    let Some(mut recipe) = app.core.recipe.find_user(recipe_id).await? else {
        return Ok(None);
    };
    scale_ingredients(&mut recipe, list.household_size);
    Ok(Some((recipe, list.state.status(recipe_id))))
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn update_step_action(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((recipe_id, direction)): Path<(String, String)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let list = imkitchen_web_shared::try_page_response!(load_list(&app, &user.id), template);
    let (slot_recipe, status) = imkitchen_web_shared::try_page_response!(opt: find_list_recipe(&app, &list, &recipe_id), template);

    let status = next_status(&direction, &status, slot_recipe.instructions.len());

    imkitchen_web_shared::try_response!(
        app.core.shopping.change_recipe_status(
            ChangeRecipeStatus {
                recipe_id: recipe_id.clone(),
                status: status.clone()
            },
            &user.id
        ),
        template
    );

    // Compute view state from the NEW status in-memory — re-reading the
    // aggregate here is unnecessary and would race with evento's async
    // snapshot update.
    let (completed_instructions, coming_instructions, current_instruction) =
        split_instructions(&slot_recipe, &status, false);

    // Ingredient list is the first screen of the cooking flow — shown while the
    // recipe is Idle, but only when it actually has in-app steps to cook.
    let show_ingredients = status.is_idle() && !slot_recipe.instructions.is_empty();
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
            show_iframe,
            show_ingredients,
            ingredient_aisles,
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/kitchen-dish.html")]
pub struct KitchenDishTemplate {
    pub entries: Vec<ListEntry>,
    pub slot_recipe: UserView,
    pub focused_status: RecipeStatus,
    pub completed_instructions: Vec<(usize, String)>,
    pub coming_instructions: Vec<(usize, String)>,
    pub current_instruction: Option<(usize, Instruction)>,
    pub cook_external: bool,
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn select_dish(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((recipe_id,)): Path<(String,)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let list = imkitchen_web_shared::try_page_response!(load_list(&app, &user.id), template);
    let Some((slot_recipe, status)) = imkitchen_web_shared::try_page_response!(
        find_list_recipe(&app, &list, &recipe_id),
        template
    ) else {
        return template.render(NotFoundTemplate).into_response();
    };

    let cards = imkitchen_web_shared::try_page_response!(
        app.core.recipe.filter_by_ids(list.state.recipe_ids.clone()),
        template
    );
    let entries = list_entries(&list.state, cards);

    let (completed_instructions, coming_instructions, current_instruction) =
        split_instructions(&slot_recipe, &status, true);

    let cook_external = cook_is_external(&app, &slot_recipe, &current_instruction).await;

    template
        .render(KitchenDishTemplate {
            entries,
            slot_recipe,
            focused_status: status,
            completed_instructions,
            coming_instructions,
            current_instruction,
            cook_external,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = tracing::field::Empty))]
pub async fn cook_page(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Path((recipe_id,)): Path<(String,)>,
) -> impl IntoResponse {
    tracing::Span::current().record("user", &user.id);

    let list = imkitchen_web_shared::try_page_response!(load_list(&app, &user.id), template);
    let (slot_recipe, status) = imkitchen_web_shared::try_page_response!(opt: find_list_recipe(&app, &list, &recipe_id), template);

    // `Idle` renders the ingredient list (first screen); `Cooking(0)` is the
    // first instruction and `Completed` the last.
    let (completed_instructions, coming_instructions, current_instruction) =
        split_instructions(&slot_recipe, &status, false);

    // Ingredient list is the first screen — shown while Idle, but only when the
    // recipe actually has in-app steps to cook.
    let show_ingredients = status.is_idle() && !slot_recipe.instructions.is_empty();
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
            show_iframe,
            show_ingredients,
            ingredient_aisles,
        })
        .into_response()
}

/// Calendar-era `/kitchen/{date}` bookmarks.
async fn legacy_kitchen_redirect() -> impl IntoResponse {
    Redirect::permanent("/")
}

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/", get(page))
        .route(
            "/kitchen/{recipe_id}/step/{direction}",
            post(update_step_action),
        )
        .route("/kitchen/{recipe_id}/select-dish", post(select_dish))
        .route("/kitchen/{recipe_id}/cook", get(cook_page))
        .route("/kitchen/{legacy}", get(legacy_kitchen_redirect))
}

#[cfg(test)]
mod tests {
    use super::next_status;
    use imkitchen_types::shopping::RecipeStatus::*;

    #[test]
    fn walks_forward_through_steps_to_completed() {
        assert_eq!(next_status("next", &Idle, 3), Cooking(0));
        assert_eq!(next_status("next", &Cooking(0), 3), Cooking(1));
        assert_eq!(next_status("next", &Cooking(1), 3), Completed);
        assert_eq!(next_status("next", &Completed, 3), Completed);
    }

    #[test]
    fn walks_back_from_completed_to_idle() {
        assert_eq!(next_status("prev", &Completed, 3), Cooking(1));
        assert_eq!(next_status("prev", &Cooking(1), 3), Cooking(0));
        assert_eq!(next_status("prev", &Cooking(0), 3), Idle);
        assert_eq!(next_status("prev", &Idle, 3), Idle);
    }

    #[test]
    fn single_step_recipes_skip_cooking() {
        assert_eq!(next_status("next", &Idle, 1), Completed);
        assert_eq!(next_status("prev", &Completed, 1), Idle);
        assert_eq!(next_status("next", &Idle, 0), Completed);
    }
}
