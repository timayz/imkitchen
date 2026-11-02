use evento::Executor;
use imkitchen_types::{recipe::Ingredient, shopping::RecipeStatus};
use std::collections::{HashMap, HashSet};

use super::merge::{Servings, merge_ingredients};

/// Current list state, computed straight from the aggregate so it is
/// immediately consistent after a command. Used to render the menu and
/// kitchen pages and to re-render them right after add/remove.
pub struct ShoppingState {
    /// Recipe ids in list order.
    pub recipe_ids: Vec<String>,
    pub ingredients: Vec<Ingredient>,
    pub checked: HashSet<String>,
    pub statuses: HashMap<String, RecipeStatus>,
    /// Household size recorded per recipe id when it was generated or added.
    pub servings: HashMap<String, u16>,
    /// Household size for recipes without a recorded one (the user's settings
    /// value at read time).
    pub default_servings: u16,
}

impl ShoppingState {
    pub fn status(&self, recipe_id: &str) -> RecipeStatus {
        self.statuses.get(recipe_id).cloned().unwrap_or_default()
    }

    pub fn contains(&self, recipe_id: &str) -> bool {
        self.recipe_ids.iter().any(|id| id == recipe_id)
    }

    /// Household size the recipe's ingredients are scaled to.
    pub fn servings(&self, recipe_id: &str) -> u16 {
        self.servings
            .get(recipe_id)
            .copied()
            .unwrap_or(self.default_servings)
    }
}

impl<E: Executor> super::Module<E> {
    /// Load the aggregate and recompute the merged ingredient list for the
    /// current recipe set. Each recipe is scaled to the household size recorded
    /// when it was generated or added; `default_household_size` (the user's
    /// settings value) covers recipes listed before servings were recorded.
    pub async fn state(
        &self,
        user_id: impl Into<String>,
        default_household_size: u16,
    ) -> anyhow::Result<ShoppingState> {
        let (recipe_ids, checked, statuses, servings) = match self.load(user_id).await? {
            Some(s) => (s.recipes, s.checked, s.statuses, s.servings),
            None => (vec![], HashSet::new(), HashMap::new(), HashMap::new()),
        };

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(recipe_ids.clone())
            .await?;
        let ingredients = merge_ingredients(
            recipe_ingredients,
            &Servings {
                per_recipe: &servings,
                default: default_household_size,
            },
        );

        Ok(ShoppingState {
            recipe_ids,
            ingredients,
            checked,
            statuses,
            servings,
            default_servings: default_household_size,
        })
    }
}
