use evento::{Executor, ProjectionAggregate};
use imkitchen_types::shopping::{Checked, Unchecked};

pub struct ToggleInput {
    pub name: String,
}

impl<E: Executor> super::Module<E> {
    pub async fn toggle(
        &self,
        input: ToggleInput,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let Some(shopping) = self.load(&request_by).await? else {
            crate::not_found!("shopping in toogle");
        };

        // The aggregate's `ingredients` is a snapshot taken when the list last
        // changed; the `shopping_recipe` projection may have lagged behind it,
        // or a recipe may have been edited since. Validate against the live
        // recipe ingredients instead, which is what the groceries view shows.
        let known = self
            .filter_recipe_ingredients_by_ids(shopping.recipes.clone())
            .await?
            .iter()
            .flat_map(|(_, ingredients)| ingredients)
            .any(|ingredient| ingredient.key() == input.name);

        if !known {
            crate::user!("ingredient not found");
        }

        let checked = shopping.checked.contains(&input.name);

        if checked {
            shopping
                .write()?
                .event(&Unchecked {
                    ingredient: input.name,
                })
                .requested_by(request_by)
                .commit(&self.executor)
                .await?;
        } else {
            shopping
                .write()?
                .event(&Checked {
                    ingredient: input.name,
                })
                .requested_by(request_by)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
