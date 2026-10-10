//! The kitchen: the user's recipe list, what to cook next from it, and the
//! step-by-step cooking screens. Pure data in, pure data out; the HTML
//! crate renders these into templates, the API serializes them.

use imkitchen_core::recipe::query::user::{RecipeCard, UserView};
use imkitchen_core::shopping::{ChangeRecipeStatus, GenerateList, Randomize, ShoppingState};
use imkitchen_types::recipe::{Ingredient, Instruction, RecipeType};
use imkitchen_types::shopping::RecipeStatus;

use crate::AppState;

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
    pub thumbnail_version: Option<String>,
    pub blur_placeholder: Option<String>,
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

/// A single grocery aisle section of a recipe's ingredient list, keyed by the
/// `shopping_<Category>` string so templates can reuse the groceries aisle
/// macros (emoji / label / accent color).
pub struct IngredientAisle {
    pub name: String,
    pub items: Vec<Ingredient>,
}

/// `(completed, coming, current)` instructions around the cooking cursor.
pub type StepView = (
    Vec<(usize, String)>,
    Vec<(usize, String)>,
    Option<(usize, Instruction)>,
);

/// The list state plus the household size it was scaled for.
pub struct ListContext {
    pub state: ShoppingState,
    pub household_size: u16,
}

pub async fn load_list(app: &AppState, user_id: &str) -> anyhow::Result<ListContext> {
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

/// Whether the "Start cooking" button should link straight to the recipe's
/// original URL (opened externally) rather than into the in-app cooking screen.
///
/// True only when the recipe has no parsed steps AND its origin refuses framing —
/// i.e. there is nothing to cook in-app. The embeddability check runs only for
/// step-less recipes (and is cached per-domain), so recipes with steps never
/// trigger a network call here.
pub async fn cook_is_external(
    app: &AppState,
    recipe: &UserView,
    current_instruction: &Option<(usize, Instruction)>,
) -> bool {
    if current_instruction.is_some() {
        return false;
    }
    match recipe.origin.as_deref() {
        Some(origin) => !origin_embeddable(app, origin).await,
        None => false,
    }
}

async fn origin_embeddable(app: &AppState, origin: &str) -> bool {
    app.core
        .recipe
        .is_origin_embeddable(origin)
        .await
        .unwrap_or(false)
}

/// Scale a recipe's ingredient quantities to the user's household size and
/// sort them by name. Shared by every kitchen screen that shows ingredients
/// (dashboard, dish preview, and the cooking screen) so they stay consistent.
pub fn scale_ingredients(recipe: &mut UserView, household_size: u16) {
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
pub fn group_ingredients_by_aisle(ingredients: &[Ingredient]) -> Vec<IngredientAisle> {
    let mut categories: std::collections::HashMap<String, Vec<Ingredient>> =
        std::collections::HashMap::new();

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

/// The list as kitchen entries, in list order: generated meals come as
/// starter, main, side, dessert, drink, sauce; manual adds follow. Ids whose
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
                    thumbnail_version: card.thumbnail_version,
                    blur_placeholder: card.blur_placeholder,
                },
            ))
        })
        .collect();
    entries.sort_by_key(|(pos, _)| *pos);
    entries.into_iter().map(|(_, e)| e).collect()
}

/// Split a recipe's instructions around the cooking cursor.
///
/// `Idle` means the recipe has not been started: the dashboard previews the
/// first step (`idle_shows_first_step`), while the cooking screen shows the
/// ingredient list instead and has no current step.
pub fn split_instructions(
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
/// and `Completed` the last. Unknown directions leave the cursor where it is.
pub fn next_status(direction: &str, current: &RecipeStatus, len: usize) -> RecipeStatus {
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

/// A recipe from the list, scaled, with its cooking status. `None` when the
/// recipe is not in the list (or no longer exists).
pub async fn find_list_recipe(
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

/// What the kitchen home shows.
pub enum Overview {
    /// The list has nothing to cook: empty, or every recipe in it was
    /// deleted. The guided tours explain what to do; the page only offers
    /// the one action that applies.
    Empty(Empty),
    /// The list, with the next recipe to cook in focus.
    List(Box<KitchenList>),
}

pub struct Empty {
    /// At least one main course in the pool: generating is possible.
    pub has_recipes: bool,
}

pub struct KitchenList {
    /// Every recipe in the list, in list order (generated meals read starter →
    /// main → side → dessert → drink → sauce).
    pub entries: Vec<ListEntry>,
    /// The recipe to cook next: the first one in the list not yet cooked.
    pub focused: UserView,
    pub focused_status: RecipeStatus,
    pub completed_count: usize,
    pub total_count: usize,
    /// Uncooked recipes (other than the focused one) with an advance-prep note.
    pub prep_ahead: Vec<ListEntry>,
    pub steps: StepView,
    /// When true, "Start cooking" opens the recipe's original URL externally
    /// instead of the in-app cooking screen. See [`cook_is_external`].
    pub cook_external: bool,
}

pub async fn overview(app: &AppState, user_id: &str) -> anyhow::Result<Overview> {
    let list = load_list(app, user_id).await?;

    let cards = if list.state.recipe_ids.is_empty() {
        vec![]
    } else {
        app.core
            .recipe
            .filter_by_ids(list.state.recipe_ids.clone())
            .await?
    };
    let entries = list_entries(&list.state, cards);

    // The recipe to cook next: the first uncooked entry whose recipe still
    // resolves, else the first one that does. A list whose recipes were all
    // deleted has nothing to show and counts as empty.
    let mut next = None;
    for entry in entries
        .iter()
        .filter(|e| !e.is_completed())
        .chain(entries.iter())
    {
        if let Some(recipe) = app.core.recipe.find_user(&entry.id).await? {
            next = Some((entry.clone(), recipe));
            break;
        }
    }
    let Some((focused_entry, mut focused)) = next else {
        return empty(app, user_id).await;
    };

    let total_count = entries.len();
    let completed_count = entries.iter().filter(|e| e.is_completed()).count();

    let focused_status = focused_entry.status.clone();
    scale_ingredients(&mut focused, list.household_size);
    let steps = split_instructions(&focused, &focused_status, true);

    let prep_ahead: Vec<ListEntry> = entries
        .iter()
        .filter(|e| !e.advance_prep.trim().is_empty() && !e.is_completed())
        .filter(|e| e.id != focused_entry.id)
        .cloned()
        .collect();

    let cook_external = cook_is_external(app, &focused, &steps.2).await;

    Ok(Overview::List(Box::new(KitchenList {
        entries,
        focused,
        focused_status,
        completed_count,
        total_count,
        prep_ahead,
        steps,
        cook_external,
    })))
}

/// The kitchen home when the list has nothing to cook: whether the pool has
/// a main course to generate from decides the action offered.
async fn empty(app: &AppState, user_id: &str) -> anyhow::Result<Overview> {
    let main_courses = app
        .core
        .shopping
        .sample_recipes(user_id, RecipeType::MainCourse)
        .await?;

    Ok(Overview::Empty(Empty {
        has_recipes: !main_courses.is_empty(),
    }))
}

/// The kitchen with another recipe of the list in focus.
pub struct Dish {
    pub entries: Vec<ListEntry>,
    pub recipe: UserView,
    pub status: RecipeStatus,
    pub steps: StepView,
    pub cook_external: bool,
}

pub async fn dish(app: &AppState, user_id: &str, recipe_id: &str) -> anyhow::Result<Option<Dish>> {
    let list = load_list(app, user_id).await?;
    let Some((recipe, status)) = find_list_recipe(app, &list, recipe_id).await? else {
        return Ok(None);
    };

    let cards = app
        .core
        .recipe
        .filter_by_ids(list.state.recipe_ids.clone())
        .await?;
    let entries = list_entries(&list.state, cards);

    let steps = split_instructions(&recipe, &status, true);
    let cook_external = cook_is_external(app, &recipe, &steps.2).await;

    Ok(Some(Dish {
        entries,
        recipe,
        status,
        steps,
        cook_external,
    }))
}

/// One screen of the cooking flow: the ingredient list while `Idle`, then one
/// instruction per step.
pub struct CookingScreen {
    pub recipe: UserView,
    pub status: RecipeStatus,
    pub steps: StepView,
    /// The recipe's origin page may be framed in place of in-app steps.
    pub origin_embeddable: bool,
    /// Render the ingredient list (grouped in `ingredient_aisles`) as the first
    /// screen of the cooking flow instead of a step.
    pub show_ingredients: bool,
    pub ingredient_aisles: Vec<IngredientAisle>,
}

impl CookingScreen {
    /// Imported recipe with no parsed steps whose origin refuses framing: there
    /// is nothing to show in-app, so the user goes straight to the original.
    pub fn external_only(&self) -> Option<&str> {
        if !self.origin_embeddable && self.recipe.instructions.is_empty() {
            self.recipe.origin.as_deref()
        } else {
            None
        }
    }
}

async fn build_cooking_screen(
    app: &AppState,
    recipe: UserView,
    status: RecipeStatus,
) -> CookingScreen {
    // `Idle` renders the ingredient list (first screen); `Cooking(0)` is the
    // first instruction and `Completed` the last.
    let steps = split_instructions(&recipe, &status, false);

    // Ingredient list is the first screen — shown while Idle, but only when the
    // recipe actually has in-app steps to cook.
    let show_ingredients = status.is_idle() && !recipe.instructions.is_empty();
    let ingredient_aisles = if show_ingredients {
        group_ingredients_by_aisle(&recipe.ingredients)
    } else {
        vec![]
    };

    let origin_embeddable = match recipe.origin.as_deref() {
        Some(origin) => origin_embeddable(app, origin).await,
        None => false,
    };

    CookingScreen {
        recipe,
        status,
        steps,
        origin_embeddable,
        show_ingredients,
        ingredient_aisles,
    }
}

pub async fn cooking_screen(
    app: &AppState,
    user_id: &str,
    recipe_id: &str,
) -> anyhow::Result<Option<CookingScreen>> {
    let list = load_list(app, user_id).await?;
    let Some((recipe, status)) = find_list_recipe(app, &list, recipe_id).await? else {
        return Ok(None);
    };
    Ok(Some(build_cooking_screen(app, recipe, status).await))
}

/// Move the cooking cursor (`"next"` / `"prev"`) and return the resulting
/// screen. The view is computed from the NEW status in memory: re-reading the
/// aggregate would race with evento's async snapshot update.
pub async fn step(
    app: &AppState,
    user_id: &str,
    recipe_id: &str,
    direction: &str,
) -> imkitchen_core::Result<Option<CookingScreen>> {
    let list = load_list(app, user_id).await?;
    let Some((recipe, status)) = find_list_recipe(app, &list, recipe_id).await? else {
        return Ok(None);
    };

    let status = next_status(direction, &status, recipe.instructions.len());

    app.core
        .shopping
        .change_recipe_status(
            ChangeRecipeStatus {
                recipe_id: recipe_id.to_owned(),
                status: status.clone(),
            },
            user_id,
        )
        .await?;

    Ok(Some(build_cooking_screen(app, recipe, status).await))
}

/// Set the cooking cursor to an absolute position and return the resulting
/// screen. `Cooking(n)` must point at an existing instruction. Nothing is
/// written when the status is unchanged, so a retried call is harmless.
pub async fn set_status(
    app: &AppState,
    user_id: &str,
    recipe_id: &str,
    status: RecipeStatus,
) -> imkitchen_core::Result<Option<CookingScreen>> {
    let list = load_list(app, user_id).await?;
    let Some((recipe, current)) = find_list_recipe(app, &list, recipe_id).await? else {
        return Ok(None);
    };

    if let RecipeStatus::Cooking(pos) = status
        && pos as usize >= recipe.instructions.len()
    {
        return Err(imkitchen_core::Error::User("step out of range".to_owned()));
    }

    if status != current {
        app.core
            .shopping
            .change_recipe_status(
                ChangeRecipeStatus {
                    recipe_id: recipe_id.to_owned(),
                    status: status.clone(),
                },
                user_id,
            )
            .await?;
    }

    Ok(Some(build_cooking_screen(app, recipe, status).await))
}

/// Replace the list with freshly picked recipes. The list is read back from
/// the aggregate, so the next overview is already up to date — no polling.
pub async fn generate(app: &AppState, user_id: &str, count: u8) -> imkitchen_core::Result<()> {
    let preferences = app.identity.meal_preferences.load(user_id).await?;

    app.core
        .shopping
        .generate(
            GenerateList {
                count,
                household_size: preferences.household_size,
                randomize: Some(Randomize {
                    cuisine_variety_weight: preferences.cuisine_variety_weight,
                    dietary_restrictions: preferences.dietary_restrictions.to_vec(),
                    recipe_types: preferences.recipe_types.to_vec(),
                }),
            },
            user_id,
        )
        .await?;

    Ok(())
}

/// Remove a recipe from the list.
pub async fn remove(app: &AppState, user_id: &str, recipe_id: &str) -> imkitchen_core::Result<()> {
    let preferences = app.identity.meal_preferences.load(user_id).await?;
    app.core
        .shopping
        .remove_recipe(recipe_id, preferences.household_size, user_id)
        .await?;
    Ok(())
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
