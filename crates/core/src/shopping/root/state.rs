use evento::Executor;
use imkitchen_types::recipe::Ingredient;
use std::collections::HashSet;

use super::merge::merge_ingredients;

/// Current shopping-list state, computed straight from the aggregate so it is
/// immediately consistent after a command (unlike the `shopping_list` read
/// model, which a background subscription updates asynchronously). Used to
/// render the groceries page and to re-render it right after add/remove.
pub struct ShoppingState {
    pub recipe_ids: Vec<String>,
    pub ingredients: Vec<Ingredient>,
    pub checked: HashSet<String>,
    pub from_date: u64,
    pub days: u8,
}

impl<E: Executor> super::Module<E> {
    /// Load the aggregate and recompute the merged ingredient list for the
    /// current recipe set, scaled to `household_size`.
    pub async fn state(
        &self,
        user_id: impl Into<String>,
        household_size: u16,
    ) -> anyhow::Result<ShoppingState> {
        let (recipe_ids, checked, from_date, days) = match self.load(user_id).await? {
            Some(s) => (
                s.recipes.into_iter().collect::<Vec<_>>(),
                s.checked,
                s.from_date,
                s.days,
            ),
            None => (vec![], HashSet::new(), 0, 0),
        };

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(recipe_ids.clone())
            .await?;
        let ingredients = merge_ingredients(recipe_ingredients, household_size);

        Ok(ShoppingState {
            recipe_ids,
            ingredients,
            checked,
            from_date,
            days,
        })
    }
}
