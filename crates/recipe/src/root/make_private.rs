use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::recipe::MadePrivate;

impl<E: Executor + Clone> super::Command<E> {
    pub async fn make_private(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let Some(recipe) = self.load(id).await? else {
            imkitchen_shared::not_found!("recipe");
        };

        let request_by = request_by.into();
        if recipe.owner_id != request_by {
            imkitchen_shared::forbidden!("not owner of recipe");
        }

        if recipe.is_shared {
            recipe
                .aggregator()?
                .event(&MadePrivate)
                .metadata(&Metadata::new(request_by))
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
