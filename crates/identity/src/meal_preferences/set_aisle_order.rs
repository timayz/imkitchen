use evento::{Executor, ProjectionAggregate};
use imkitchen_types::meal_preferences::{AisleOrderChanged, Changed, RecipeTypesChanged};
use imkitchen_types::recipe::IngredientCategory;

impl<E: Executor> super::Module<E> {
    /// Sets the order aisles appear in on the groceries page. `aisles` may be
    /// partial or repeated: the first occurrence of each wins and the missing
    /// ones follow in default order, so the stored order is always complete.
    pub async fn set_aisle_order(
        &self,
        id: impl Into<String>,
        aisles: Vec<IngredientCategory>,
    ) -> imkitchen_core::Result<()> {
        let id = id.into();
        let preferences = self.load(&id).await?;
        let aisles = IngredientCategory::complete_aisle_order(&aisles);

        // A projection row starts zeroed, so a stream whose first event is
        // `AisleOrderChanged` would load as household 0 / no courses instead of
        // the defaults `load` hands out when there is no stream at all. Seed
        // the defaults first on a fresh stream.
        let fresh = preferences.aggregate_version == 0;
        let mut write = preferences.write()?;
        if fresh {
            write
                .event(&Changed {
                    household_size: preferences.household_size,
                    dietary_restrictions: preferences.dietary_restrictions.clone(),
                    cuisine_variety_weight: preferences.cuisine_variety_weight,
                })
                .event(&RecipeTypesChanged {
                    recipe_types: preferences.recipe_types.clone(),
                });
        }
        write
            .event(&AisleOrderChanged { aisles })
            .requested_by(id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
