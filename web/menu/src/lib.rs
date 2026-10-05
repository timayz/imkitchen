//! The user's recipe list, in two tabs: the recipes themselves (generate,
//! add, remove) and the aisle-grouped groceries they need.

use axum::{
    extract::{Json, Path, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use imkitchen_core::recipe::query::user::RecipeCard;
use imkitchen_core::shopping::{GenerateList, Randomize, ToggleInput};
use imkitchen_types::recipe::{Ingredient, IngredientUnitFormat, RecipeType};
use imkitchen_types::shopping::RecipeStatus;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use imkitchen_web_shared::{
    auth::AuthUser,
    state::AppState,
    template::{Template, filters},
};

pub fn routes() -> axum::Router<imkitchen_web_shared::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/menu", get(page))
        .route("/menu/groceries", get(groceries_page))
        .route("/menu/generate", get(generate_modal).post(generate_action))
        .route("/menu/toggle", post(toggle_action))
        .route("/menu/recipe/{id}/remove", post(remove_recipe_action))
        // Calendar-era `/menu/{date}` bookmarks.
        .route("/menu/{legacy}", get(legacy_menu_redirect))
        .route("/groceries", get(legacy_groceries_redirect))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuTab {
    Recipes,
    Groceries,
}

/// A recipe in the list together with its cooking status.
pub struct ListRecipe {
    pub card: RecipeCard,
    pub status: RecipeStatus,
}

pub struct AisleSection {
    pub name: String,
    pub items: Vec<Ingredient>,
    pub checked: usize,
    pub total: usize,
    pub done: bool,
    pub pct: usize,
}

#[derive(askama::Template)]
#[template(path = "menu.html")]
pub struct MenuTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub tab: MenuTab,
    pub recipes: Vec<ListRecipe>,
    pub cooked_count: usize,
    pub checked: HashSet<String>,
    pub aisles: Vec<AisleSection>,
    /// Index into `aisles` where the right desktop column starts (aisles are
    /// split into two columns balanced by item count).
    pub split_at: usize,
    pub total_items: usize,
    pub checked_items: usize,
    pub progress_pct: usize,
}

impl Default for MenuTemplate {
    fn default() -> Self {
        Self {
            current_path: "menu".to_owned(),
            user: AuthUser::default(),
            tab: MenuTab::Recipes,
            recipes: vec![],
            cooked_count: 0,
            checked: HashSet::default(),
            aisles: vec![],
            split_at: 0,
            total_items: 0,
            checked_items: 0,
            progress_pct: 0,
        }
    }
}

/// Recipes-tab fragment swapped in via twinspark when a recipe is removed.
#[derive(askama::Template)]
#[template(path = "partials/menu-recipes.html")]
pub struct MenuRecipesTemplate {
    pub recipes: Vec<ListRecipe>,
    pub cooked_count: usize,
}

#[derive(askama::Template)]
#[template(path = "partials/menu-generate-modal.html")]
pub struct GenerateModalTemplate;

/// Everything both tabs need, derived from the persisted list.
pub struct ListView {
    pub recipes: Vec<ListRecipe>,
    pub cooked_count: usize,
    pub checked: HashSet<String>,
    pub aisles: Vec<AisleSection>,
    pub split_at: usize,
    pub total_items: usize,
    pub checked_items: usize,
    pub progress_pct: usize,
}

async fn build_view(app: &AppState, user_id: &str) -> anyhow::Result<ListView> {
    // Read straight from the aggregate (immediately consistent) so a re-render
    // right after add/remove/generate never shows the pre-change list.
    let household_size = app
        .identity
        .meal_preferences
        .load(user_id)
        .await?
        .household_size;
    let state = app.core.shopping.state(user_id, household_size).await?;

    let cards = app
        .core
        .recipe
        .filter_by_ids(state.recipe_ids.clone())
        .await?;
    let recipes: Vec<ListRecipe> = order_by_list(&state.recipe_ids, cards)
        .into_iter()
        .map(|card| ListRecipe {
            status: state.status(&card.id),
            card,
        })
        .collect();
    let cooked_count = recipes.iter().filter(|r| r.status.is_completed()).count();

    Ok(ListView {
        recipes,
        cooked_count,
        ..grocery_view(&state.ingredients, state.checked)
    })
}

/// Aisle sections, counts and the column split for an ingredient list.
pub fn grocery_view(ingredients: &[Ingredient], checked: HashSet<String>) -> ListView {
    let categories: Vec<(String, Vec<Ingredient>)> = to_categories(ingredients);

    let total_items: usize = categories.iter().map(|(_, items)| items.len()).sum();
    // Count only keys still on the list: the aggregate keeps checks for
    // ingredients a removed recipe took away.
    let checked_items = ingredients
        .iter()
        .filter(|i| checked.contains(&i.key()))
        .count();
    let progress_pct = (checked_items * 100).checked_div(total_items).unwrap_or(0);

    let aisles: Vec<AisleSection> = categories
        .into_iter()
        .map(|(name, items)| {
            let total = items.len();
            let checked_count = items.iter().filter(|i| checked.contains(&i.key())).count();
            let pct = (checked_count * 100).checked_div(total).unwrap_or(0);
            AisleSection {
                name,
                items,
                checked: checked_count,
                total,
                done: total > 0 && checked_count == total,
                pct,
            }
        })
        .collect();

    let split_at = balanced_split(&aisles);

    ListView {
        recipes: vec![],
        cooked_count: 0,
        checked,
        aisles,
        split_at,
        total_items,
        checked_items,
        progress_pct,
    }
}

/// `filter_by_ids` returns rows in no particular order; put them back in list
/// order (ids without a row are skipped).
fn order_by_list(ids: &[String], cards: Vec<RecipeCard>) -> Vec<RecipeCard> {
    let mut by_id: HashMap<String, RecipeCard> =
        cards.into_iter().map(|c| (c.id.clone(), c)).collect();
    ids.iter().filter_map(|id| by_id.remove(id)).collect()
}

/// Choose where the right desktop column starts. Aisles keep their route order;
/// the split is the contiguous point that most evenly divides the total item
/// count between the two columns. E.g. counts `[2, 54, 6, 4, 39, 2, 1]` split
/// after index 2 → `[2, 54]` (56) and `[6, 4, 39, 2, 1]` (52). Both columns are
/// always non-empty (for 2+ aisles).
fn balanced_split(aisles: &[AisleSection]) -> usize {
    let n = aisles.len();
    if n <= 1 {
        return n;
    }
    let total: usize = aisles.iter().map(|a| a.total).sum();
    let mut left = 0usize;
    let mut best_split = 1;
    let mut best_diff = usize::MAX;
    // Consider splitting after each aisle except the last, so both columns are
    // non-empty; the split index is `i + 1`.
    for (i, aisle) in aisles[..n - 1].iter().enumerate() {
        left += aisle.total;
        let diff = left.abs_diff(total - left);
        if diff < best_diff {
            best_diff = diff;
            best_split = i + 1;
        }
    }
    best_split
}

fn to_categories(ingredients: &[Ingredient]) -> Vec<(String, Vec<Ingredient>)> {
    let mut categories = HashMap::new();
    let mut ingredients = ingredients.to_vec();
    ingredients.sort_by_key(|i| i.name.to_owned());

    for ingredient in ingredients.iter() {
        match &ingredient.category {
            Some(c) => {
                let entry = categories.entry(format!("shopping_{c}")).or_insert(vec![]);
                entry.push(ingredient.clone());
            }
            _ => {
                let entry = categories
                    .entry("shopping_Unknown".to_owned())
                    .or_insert(vec![]);
                entry.push(ingredient.clone());
            }
        };
    }

    let mut categories = categories
        .into_iter()
        .collect::<Vec<(String, Vec<Ingredient>)>>();

    categories.sort_by_key(|(k, _)| k.to_owned());

    categories
}

fn render_page(
    template: Template,
    user: AuthUser,
    tab: MenuTab,
    view: ListView,
) -> axum::response::Response {
    template
        .render(MenuTemplate {
            user,
            tab,
            recipes: view.recipes,
            cooked_count: view.cooked_count,
            checked: view.checked,
            aisles: view.aisles,
            split_at: view.split_at,
            total_items: view.total_items,
            checked_items: view.checked_items,
            progress_pct: view.progress_pct,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let view = imkitchen_web_shared::try_page_response!(build_view(&app, &user.id), template);
    render_page(template, user, MenuTab::Recipes, view)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn groceries_page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let view = imkitchen_web_shared::try_page_response!(build_view(&app, &user.id), template);
    render_page(template, user, MenuTab::Groceries, view)
}

async fn legacy_menu_redirect() -> impl IntoResponse {
    Redirect::permanent("/menu")
}

async fn legacy_groceries_redirect() -> impl IntoResponse {
    Redirect::permanent("/menu/groceries")
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn remove_recipe_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let preferences = imkitchen_web_shared::try_response!(anyhow:
        app.identity.meal_preferences.load(&user.id),
        template
    );
    imkitchen_web_shared::try_response!(
        app.core
            .shopping
            .remove_recipe(&id, preferences.household_size, &user.id),
        template
    );

    let view = imkitchen_web_shared::try_response!(anyhow: build_view(&app, &user.id), template);

    template
        .render(MenuRecipesTemplate {
            recipes: view.recipes,
            cooked_count: view.cooked_count,
        })
        .into_response()
}

#[derive(Deserialize, Default, Clone)]
pub struct ToggleJson {
    pub name: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn toggle_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Json(input): Json<ToggleJson>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(
        app.core
            .shopping
            .toggle(ToggleInput { name: input.name }, &user.id),
        template
    );

    "<div></div>".into_response()
}

pub async fn generate_modal(template: Template, _user: AuthUser) -> impl IntoResponse {
    template.render(GenerateModalTemplate)
}

#[derive(Deserialize, Debug)]
pub struct GenerateForm {
    pub count: u8,
}

/// Replace the list with freshly picked recipes, then send the browser to the
/// Recipes tab. The list is read back from the aggregate, so the redirected
/// page is already up to date — no polling needed.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn generate_action(
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Form(input): Form<GenerateForm>,
) -> impl IntoResponse {
    let preferences = imkitchen_web_shared::try_response!(anyhow:
        app.identity.meal_preferences.load(&user.id),
        template
    );

    imkitchen_web_shared::try_response!(
        app.core.shopping.generate(
            GenerateList {
                count: input.count,
                household_size: preferences.household_size,
                randomize: Some(Randomize {
                    cuisine_variety_weight: preferences.cuisine_variety_weight,
                    dietary_restrictions: preferences.dietary_restrictions.to_vec(),
                    recipe_types: preferences.recipe_types.to_vec(),
                }),
            },
            &user.id
        ),
        template
    );

    Redirect::to("/menu").into_response()
}

#[cfg(test)]
mod tests {
    use super::{AisleSection, balanced_split};

    fn aisle(total: usize) -> AisleSection {
        AisleSection {
            name: format!("a{total}"),
            items: vec![],
            checked: 0,
            total,
            done: false,
            pct: 0,
        }
    }

    fn split(totals: Vec<usize>) -> (usize, usize, usize) {
        let aisles: Vec<AisleSection> = totals.into_iter().map(aisle).collect();
        let split_at = balanced_split(&aisles);
        let left: usize = aisles[..split_at].iter().map(|a| a.total).sum();
        let right: usize = aisles[split_at..].iter().map(|a| a.total).sum();
        (split_at, left, right)
    }

    #[test]
    fn contiguous_split_balances_item_counts() {
        // [2, 54, 6, 4, 39, 2, 1] → after index 2: [2,54]=56 and rest=52.
        let (split_at, left, right) = split(vec![2, 54, 6, 4, 39, 2, 1]);
        assert_eq!(split_at, 2);
        assert_eq!((left, right), (56, 52));
    }

    #[test]
    fn even_sizes_split_in_the_middle() {
        let (split_at, left, right) = split(vec![3, 3, 3, 3]);
        assert_eq!(split_at, 2);
        assert_eq!((left, right), (6, 6));
    }

    #[test]
    fn keeps_both_columns_non_empty() {
        let (split_at, _, _) = split(vec![10, 1]);
        assert_eq!(split_at, 1);
    }
}
