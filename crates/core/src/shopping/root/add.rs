use evento::{Executor, ProjectionAggregate};
use imkitchen_types::shopping::{RecipeAdded, ServingsSet};

use super::{
    check_household_size,
    merge::{Servings, merge_ingredients},
};

impl<E: Executor> super::Module<E> {
    /// Manually add a single recipe to the user's list for `household_size`
    /// people, recomputing the merged ingredient list for the new recipe set.
    ///
    /// Adding a recipe already in the list at the same size is a no-op; at a
    /// different size it only rescales that recipe (`ServingsSet`). The recipe
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
        check_household_size(household_size)?;

        if !self.recipe_exists(&recipe_id).await? {
            crate::not_found!("recipe");
        }

        let shopping = self.load_or_empty(&request_by).await?;
        let servings = ServingsSet {
            recipe_ids: vec![recipe_id.clone()],
            household_size,
        };

        if shopping.recipes.contains(&recipe_id) {
            if shopping.servings.get(&recipe_id) == Some(&household_size) {
                return Ok(());
            }
            shopping
                .write()?
                .event(&servings)
                .requested_by(request_by)
                .commit(&self.executor)
                .await?;
            return Ok(());
        }

        let mut recipe_ids = shopping.recipes.clone();
        recipe_ids.push(recipe_id.to_owned());

        let mut per_recipe = shopping.servings.clone();
        per_recipe.insert(recipe_id.clone(), household_size);
        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(recipe_ids.clone())
            .await?;
        let ingredients = merge_ingredients(
            recipe_ingredients,
            &Servings {
                per_recipe: &per_recipe,
                default: household_size,
            },
        );

        shopping
            .write()?
            .event(&RecipeAdded {
                recipe_id,
                recipe_ids,
                ingredients,
            })
            .event(&servings)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
