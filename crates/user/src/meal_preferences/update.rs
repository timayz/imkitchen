use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::{recipe::DietaryRestriction, user::meal_preferences::Changed};
use validator::Validate;

#[derive(Validate)]
pub struct UpdateInput {
    #[validate(range(min = 1))]
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    #[validate(range(min = 0.1, max = 1.0))]
    pub cuisine_variety_weight: f32,
}

impl<E: Executor> super::Command<E> {
    pub async fn update(
        &self,
        id: impl Into<String>,
        input: UpdateInput,
    ) -> imkitchen_shared::Result<()> {
        input.validate()?;

        let id = id.into();
        let preferences = self.load(&id).await?;

        preferences
            .aggregator()?
            .event(&Changed {
                dietary_restrictions: input.dietary_restrictions,
                household_size: input.household_size,
                cuisine_variety_weight: input.cuisine_variety_weight,
            })
            .requested_by(id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
