use evento::{Executor, ProjectionAggregate};
use imkitchen_types::shopping::RecipeAdded;

use super::merge::merge_ingredients;

impl<E: Executor> super::Module<E> {
    /// Manually add a single recipe to the user's shopping list, recomputing the
    /// merged ingredient list for the new recipe set.
    ///
    /// Idempotent: adding a recipe already in the list is a no-op. The recipe
    /// must exist in the `shopping_recipe` projection; ownership is enforced by
    /// the caller (web layer), so shared recipes can be added too.
    pub async fn add_recipe(
        &self,
        recipe_id: impl Into<String>,
        household_size: u16,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let recipe_id = recipe_id.into();

        if !self.recipe_exists(&recipe_id).await? {
            crate::not_found!("recipe");
        }

        let shopping = self
            .load(&request_by)
            .await?
            .unwrap_or_else(|| super::Shopping {
                user_id: request_by.to_owned(),
                checked: Default::default(),
                ingredients: Default::default(),
                recipes: Default::default(),
                cursor: Default::default(),
                from_date: 0,
                days: 0,
                generated_at: 0,
            });

        if shopping.recipes.contains(&recipe_id) {
            return Ok(());
        }

        let mut recipe_ids: Vec<String> = shopping.recipes.iter().cloned().collect();
        recipe_ids.push(recipe_id.to_owned());

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(recipe_ids.clone())
            .await?;
        let ingredients = merge_ingredients(recipe_ingredients, household_size);

        shopping
            .write()?
            .event(&RecipeAdded {
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
