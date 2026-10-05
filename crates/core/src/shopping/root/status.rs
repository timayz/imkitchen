use evento::{Executor, ProjectionAggregate};
use imkitchen_types::shopping::{RecipeStatus, RecipeStatusChanged};

pub struct ChangeRecipeStatus {
    pub recipe_id: String,
    pub status: RecipeStatus,
}

impl<E: Executor> super::Module<E> {
    /// Move a recipe's cooking cursor. The recipe must be in the user's list.
    pub async fn change_recipe_status(
        &self,
        input: ChangeRecipeStatus,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();

        let shopping = match self.load(&request_by).await? {
            Some(shopping) if shopping.recipes.contains(&input.recipe_id) => shopping,
            _ => crate::not_found!("recipe"),
        };

        shopping
            .write()?
            .event(&RecipeStatusChanged {
                recipe_id: input.recipe_id,
                status: input.status,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
