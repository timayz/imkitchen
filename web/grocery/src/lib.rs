//! The aisle-grouped groceries needed by every recipe in the user's list.

use axum::{
    extract::{Json, State},
    response::{IntoResponse, Redirect},
};
use imkitchen_core::shopping::ToggleInput;
use imkitchen_types::recipe::{Ingredient, IngredientUnitFormat};
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
        .route("/groceries", get(page))
        .route("/groceries/toggle", post(toggle_action))
        // The list itself lives on the kitchen page; `/menu` is kept for old
        // bookmarks (and the calendar-era `/menu/{date}`).
        .route("/menu", get(legacy_menu_redirect))
        .route("/menu/{legacy}", get(legacy_menu_redirect))
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
#[template(path = "groceries.html")]
pub struct GroceriesTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub recipe_count: usize,
    pub checked: HashSet<String>,
    pub aisles: Vec<AisleSection>,
    /// Index into `aisles` where the right desktop column starts (aisles are
    /// split into two columns balanced by item count).
    pub split_at: usize,
    pub total_items: usize,
    pub checked_items: usize,
    pub progress_pct: usize,
}

impl Default for GroceriesTemplate {
    fn default() -> Self {
        Self {
            current_path: "groceries".to_owned(),
            user: AuthUser::default(),
            recipe_count: 0,
            checked: HashSet::default(),
            aisles: vec![],
            split_at: 0,
            total_items: 0,
            checked_items: 0,
            progress_pct: 0,
        }
    }
}

/// Aisle sections, counts and the column split for an ingredient list.
pub struct GroceryView {
    pub checked: HashSet<String>,
    pub aisles: Vec<AisleSection>,
    pub split_at: usize,
    pub total_items: usize,
    pub checked_items: usize,
    pub progress_pct: usize,
}

pub fn grocery_view(ingredients: &[Ingredient], checked: HashSet<String>) -> GroceryView {
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

    GroceryView {
        checked,
        aisles,
        split_at,
        total_items,
        checked_items,
        progress_pct,
    }
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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    // Read straight from the aggregate (immediately consistent) so a render
    // right after a change never shows the previous list.
    let household_size = imkitchen_web_shared::try_page_response!(
        app.identity.meal_preferences.load(&user.id),
        template
    )
    .household_size;
    let state = imkitchen_web_shared::try_page_response!(
        app.core.shopping.state(&user.id, household_size),
        template
    );
    let view = grocery_view(&state.ingredients, state.checked);

    template
        .render(GroceriesTemplate {
            user,
            recipe_count: state.recipe_ids.len(),
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

async fn legacy_menu_redirect() -> impl IntoResponse {
    Redirect::permanent("/")
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
