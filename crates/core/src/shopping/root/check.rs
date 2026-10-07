use evento::{Executor, ProjectionAggregate};
use imkitchen_types::shopping::{Checked, Unchecked};

use super::Shopping;

pub struct SetCheckedInput {
    pub name: String,
    pub checked: bool,
}

impl<E: Executor> super::Module<E> {
    /// Absolute form of [`toggle`](Self::toggle): check or uncheck one
    /// ingredient. A no-op when it is already in the requested state, so a
    /// client may safely retry it (offline queues replay it).
    pub async fn set_checked(
        &self,
        input: SetCheckedInput,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let shopping = self.load_for_check(&request_by, &input.name).await?;

        if shopping.checked.contains(&input.name) == input.checked {
            return Ok(());
        }

        self.write_checked(shopping, input.name, input.checked, request_by)
            .await
    }

    /// The user's list, after checking that `name` is an ingredient of the
    /// recipes currently in it.
    pub(super) async fn load_for_check(
        &self,
        user_id: &str,
        name: &str,
    ) -> crate::Result<Shopping> {
        let Some(shopping) = self.load(user_id).await? else {
            crate::not_found!("shopping in check");
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
            .any(|ingredient| ingredient.key() == name);

        if !known {
            crate::user!("ingredient not found");
        }

        Ok(shopping)
    }

    pub(super) async fn write_checked(
        &self,
        shopping: Shopping,
        name: String,
        checked: bool,
        request_by: String,
    ) -> crate::Result<()> {
        let mut write = shopping.write()?;
        if checked {
            write.event(&Checked { ingredient: name });
        } else {
            write.event(&Unchecked { ingredient: name });
        }
        write
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
