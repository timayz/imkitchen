use evento::Executor;
use imkitchen_types::{recipe::Ingredient, shopping::RecipeStatus};
use std::collections::{HashMap, HashSet};

use super::merge::merge_ingredients;

/// Current list state, computed straight from the aggregate so it is
/// immediately consistent after a command. Used to render the menu and
/// kitchen pages and to re-render them right after add/remove.
pub struct ShoppingState {
    /// Recipe ids in list order.
    pub recipe_ids: Vec<String>,
    pub ingredients: Vec<Ingredient>,
    pub checked: HashSet<String>,
    pub statuses: HashMap<String, RecipeStatus>,
}

impl ShoppingState {
    pub fn status(&self, recipe_id: &str) -> RecipeStatus {
        self.statuses.get(recipe_id).cloned().unwrap_or_default()
    }

    pub fn contains(&self, recipe_id: &str) -> bool {
        self.recipe_ids.iter().any(|id| id == recipe_id)
    }
}

impl<E: Executor> super::Module<E> {
    /// Load the aggregate and recompute the merged ingredient list for the
    /// current recipe set, scaled to `household_size`.
    pub async fn state(
        &self,
        user_id: impl Into<String>,
        household_size: u16,
    ) -> anyhow::Result<ShoppingState> {
        let (recipe_ids, checked, statuses) = match self.load(user_id).await? {
            Some(s) => (s.recipes, s.checked, s.statuses),
            None => (vec![], HashSet::new(), HashMap::new()),
        };

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(recipe_ids.clone())
            .await?;
        let ingredients = merge_ingredients(recipe_ingredients, household_size);

        Ok(ShoppingState {
            recipe_ids,
            ingredients,
            checked,
            statuses,
        })
    }
}
