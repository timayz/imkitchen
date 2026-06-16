use evento::{Executor, ProjectionAggregate};
use imkitchen_types::user_profile::Changed;
use validator::Validate;

#[derive(Validate)]
pub struct UpdateInput {
    #[validate(length(max = 500))]
    pub description: String,
}

impl<E: Executor> super::Module<E> {
    pub async fn update(
        &self,
        id: impl Into<String>,
        input: UpdateInput,
    ) -> imkitchen_core::Result<()> {
        input.validate()?;

        let id = id.into();
        let profile = self.load(&id).await?;

        profile
            .write()?
            .event(&Changed {
                description: input.description,
            })
            .requested_by(id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
