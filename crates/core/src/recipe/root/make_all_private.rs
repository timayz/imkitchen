use evento::{Executor, ProjectionAggregator};
use imkitchen_types::recipe_share::AllMadePrivate;

impl<E: Executor + Clone> super::Module<E> {
    pub async fn make_all_private(
        &self,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();

        let share = self.load_share(&request_by).await?;

        share
            .aggregator()?
            .event(&AllMadePrivate)
            .requested_by(&request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
