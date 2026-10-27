use evento::{Aggregate, Executor, ProjectionAggregate, ProjectionCursor};
use imkitchen_types::dish::{self, Deleted};

impl<E: Executor> super::Module<E> {
    pub async fn delete(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let id = id.into();
        let Some(dish) = self.load(&id).await? else {
            crate::not_found!("dish");
        };

        let request_by = request_by.into();
        if dish.owner_id != request_by {
            crate::forbidden!("not owner of dish");
        }

        dish.write()?
            .event(&Deleted)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        self.executor
            .delete_snapshot(
                dish::Dish::aggregate_type().to_owned(),
                super::Dish::projection_name().to_owned(),
                id,
            )
            .await?;

        Ok(())
    }
}
