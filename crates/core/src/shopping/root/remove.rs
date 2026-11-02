use evento::{Executor, ProjectionAggregate};
use imkitchen_types::shopping::RecipeRemoved;

use super::merge::{Servings, merge_ingredients};

impl<E: Executor> super::Module<E> {
    /// Remove a recipe from the user's list, recomputing the merged
    /// ingredient list for the remaining recipe set. When the set becomes empty
    /// the list is left with no ingredients. `default_household_size` scales
    /// recipes that have no recorded servings of their own.
    pub async fn remove_recipe(
        &self,
        recipe_id: impl Into<String>,
        default_household_size: u16,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let recipe_id = recipe_id.into();

        let shopping = match self.load(&request_by).await? {
            Some(shopping) if shopping.recipes.contains(&recipe_id) => shopping,
            _ => crate::not_found!("recipe"),
        };

        let recipe_ids: Vec<String> = shopping
            .recipes
            .iter()
            .filter(|id| *id != &recipe_id)
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
            .event(&RecipeRemoved {
                recipe_id,
                recipe_ids,
                ingredients,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
