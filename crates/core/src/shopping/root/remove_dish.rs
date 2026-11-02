use evento::{Executor, ProjectionAggregate};
use imkitchen_types::shopping::DishRemoved;

use super::merge::{Servings, merge_ingredients};

impl<E: Executor> super::Module<E> {
    /// Remove every recipe of a dish group from the user's list in one write,
    /// recomputing the merged ingredient list for the remaining recipes. The
    /// group is the list's own (the dish may have been edited or deleted since
    /// it was placed), so courses generated around it stay.
    /// `default_household_size` scales recipes that have no recorded servings.
    pub async fn remove_dish(
        &self,
        dish_id: impl Into<String>,
        default_household_size: u16,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let dish_id = dish_id.into();

        let shopping = self.load_or_empty(&request_by).await?;
        let Some(group) = shopping.dish(&dish_id) else {
            crate::not_found!("dish");
        };

        let recipe_ids: Vec<String> = shopping
            .recipes
            .iter()
            .filter(|id| !group.recipe_ids.contains(id))
            .cloned()
            .collect();

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(recipe_ids.clone())
            .await?;
        let ingredients = merge_ingredients(
            recipe_ingredients,
            &Servings {
                per_recipe: &shopping.servings,
                default: default_household_size,
            },
        );

        shopping
            .write()?
            .event(&DishRemoved {
                dish_id,
                recipe_ids,
                ingredients,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
