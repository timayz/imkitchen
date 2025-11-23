use evento::{Executor, LoadResult};
use imkitchen_recipe::DietaryRestriction;
use imkitchen_shared::Metadata;
use sqlx::SqlitePool;
use validator::Validate;

use super::{Updated, UserMealPreferences};

#[derive(Validate)]
pub struct UpdateInput {
    #[validate(range(min = 1))]
    pub household_size: u8,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub cuisine_variety_weight: f32,
}

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
    ) -> Result<LoadResult<UserMealPreferences>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn load_optional(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<UserMealPreferences>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }

    pub async fn update(
        &self,
        input: UpdateInput,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        input.validate()?;
        let user_id = metadata.trigger_by()?;

        evento::save::<UserMealPreferences>(user_id)
            .data(&Updated {
                dietary_restrictions: input.dietary_restrictions,
                household_size: input.household_size,
                cuisine_variety_weight: input.cuisine_variety_weight,
            })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}
