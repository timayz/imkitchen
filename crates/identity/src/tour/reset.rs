use evento::{Executor, ProjectionAggregate};
use imkitchen_types::tour::Reset;

impl<E: Executor> super::Module<E> {
    /// Every tour pending again; a no-op when nothing was recorded.
    pub async fn reset(&self, id: impl Into<String>) -> imkitchen_core::Result<()> {
        let id = id.into();
        let tour = self.load(&id).await?;
        if tour.tours.is_empty() {
            return Ok(());
        }

        tour.write()?
            .event(&Reset)
            .requested_by(id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
