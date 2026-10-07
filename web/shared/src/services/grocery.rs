//! The aisle-grouped groceries needed by every recipe in the user's list.

use std::collections::{HashMap, HashSet};

use imkitchen_core::shopping::{SetCheckedInput, ToggleInput};
use imkitchen_types::recipe::{Ingredient, IngredientCategory};

use crate::AppState;

pub struct AisleSection {
    /// `shopping_<Category>`, the i18n key of the aisle label.
    pub name: String,
    pub items: Vec<Ingredient>,
    pub checked: usize,
    pub total: usize,
    pub done: bool,
    pub pct: usize,
}

/// Aisle sections, counts and the column split for an ingredient list.
pub struct GroceryView {
    pub checked: HashSet<String>,
    pub aisles: Vec<AisleSection>,
    /// Index into `aisles` where the right desktop column starts (aisles are
    /// split into two columns balanced by item count).
    pub split_at: usize,
    pub total_items: usize,
    pub checked_items: usize,
    pub progress_pct: usize,
}

/// `order` is the user's aisle order ([`IngredientCategory::complete_aisle_order`]
/// guarantees it is complete); ingredients without a category come last.
pub fn grocery_view(
    ingredients: &[Ingredient],
    checked: HashSet<String>,
    order: &[IngredientCategory],
) -> GroceryView {
    let categories: Vec<(String, Vec<Ingredient>)> = to_categories(ingredients, order);

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

/// Choose where the right desktop column starts. Aisles keep the user's order;
/// the split is the contiguous point that most evenly divides the total item
/// count between the two columns. E.g. counts `[2, 54, 6, 4, 39, 2, 1]` split
/// after index 2 → `[2, 54]` (56) and `[6, 4, 39, 2, 1]` (52). Both columns are
/// always non-empty (for 2+ aisles).
pub fn balanced_split(aisles: &[AisleSection]) -> usize {
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

/// The aisle key of an ingredient without a category.
pub const UNKNOWN_AISLE: &str = "shopping_Unknown";

fn to_categories(
    ingredients: &[Ingredient],
    order: &[IngredientCategory],
) -> Vec<(String, Vec<Ingredient>)> {
    let mut categories: HashMap<String, Vec<Ingredient>> = HashMap::new();
    let mut ingredients = ingredients.to_vec();
    ingredients.sort_by_key(|i| i.name.to_owned());

    for ingredient in ingredients.iter() {
        let key = match &ingredient.category {
            Some(c) => c.aisle_key(),
            None => UNKNOWN_AISLE.to_owned(),
        };
        categories.entry(key).or_default().push(ingredient.clone());
    }

    let mut categories = categories
        .into_iter()
        .collect::<Vec<(String, Vec<Ingredient>)>>();

    categories.sort_by_key(|(k, _)| aisle_rank(k, order));

    categories
}

/// Position of an aisle key in the user's order; unknown keys (including
/// [`UNKNOWN_AISLE`]) sort last.
fn aisle_rank(key: &str, order: &[IngredientCategory]) -> usize {
    order
        .iter()
        .position(|c| c.aisle_key() == key)
        .unwrap_or(usize::MAX)
}

/// The groceries page data: the list's recipe count plus the aisle view.
pub struct Groceries {
    pub recipe_count: usize,
    pub view: GroceryView,
}

/// Reads straight from the aggregate (immediately consistent) so a render
/// right after a change never shows the previous list.
pub async fn load(app: &AppState, user_id: &str) -> anyhow::Result<Groceries> {
    let preferences = app.identity.meal_preferences.load(user_id).await?;
    let state = app
        .core
        .shopping
        .state(user_id, preferences.household_size)
        .await?;
    let view = grocery_view(&state.ingredients, state.checked, &preferences.aisle_order);
    Ok(Groceries {
        recipe_count: state.recipe_ids.len(),
        view,
    })
}

/// Check or uncheck one ingredient by its [`Ingredient::key`].
pub async fn toggle(app: &AppState, user_id: &str, key: String) -> imkitchen_core::Result<()> {
    app.core
        .shopping
        .toggle(ToggleInput { name: key }, user_id)
        .await?;
    Ok(())
}

/// Set one ingredient's checked state by its [`Ingredient::key`]. Absolute,
/// so a client that retries (an offline queue) cannot flip it back.
pub async fn set_checked(
    app: &AppState,
    user_id: &str,
    key: String,
    checked: bool,
) -> imkitchen_core::Result<()> {
    app.core
        .shopping
        .set_checked(SetCheckedInput { name: key, checked }, user_id)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use imkitchen_types::recipe::{Ingredient, IngredientCategory};

    use super::{AisleSection, UNKNOWN_AISLE, balanced_split, grocery_view};

    fn ingredient(name: &str, category: Option<IngredientCategory>) -> Ingredient {
        Ingredient {
            name: name.to_owned(),
            quantity: 1,
            unit: None,
            category,
        }
    }

    fn aisle_names(order: &[IngredientCategory]) -> Vec<String> {
        use IngredientCategory::*;
        let ingredients = vec![
            ingredient("Salt", None),
            ingredient("Milk", Some(DairyAndEggs)),
            ingredient("Bread", Some(Bakery)),
            ingredient("Apples", Some(FruitsAndVegetables)),
        ];
        grocery_view(&ingredients, HashSet::new(), order)
            .aisles
            .into_iter()
            .map(|a| a.name)
            .collect()
    }

    #[test]
    fn aisles_follow_the_given_order_with_unknown_last() {
        use IngredientCategory::*;
        let order = IngredientCategory::complete_aisle_order(&[Bakery, DairyAndEggs]);
        assert_eq!(
            aisle_names(&order),
            vec![
                "shopping_Bakery",
                "shopping_DairyAndEggs",
                "shopping_FruitsAndVegetables",
                UNKNOWN_AISLE,
            ]
        );
    }

    #[test]
    fn default_order_is_the_store_walk() {
        assert_eq!(
            aisle_names(IngredientCategory::DEFAULT_AISLE_ORDER),
            vec![
                "shopping_FruitsAndVegetables",
                "shopping_DairyAndEggs",
                "shopping_Bakery",
                UNKNOWN_AISLE,
            ]
        );
    }

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
