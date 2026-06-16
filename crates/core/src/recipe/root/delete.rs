use evento::{Aggregate, Executor, ProjectionAggregate};
use imkitchen_types::recipe::{self, Deleted};

impl<E: Executor> super::Module<E> {
    pub async fn delete(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let id = id.into();
        let Some(recipe) = self.load(&id).await? else {
            crate::not_found!("recipe");
        };

        let request_by = request_by.into();
        if recipe.owner_id != request_by {
            crate::forbidden!("not owner of recipe");
        }

        recipe
            .write()?
            .event(&Deleted)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        self.executor
            .delete_snapshot(recipe::Recipe::aggregate_type().to_owned(), id)
            .await?;

        Ok(())
    }
}
