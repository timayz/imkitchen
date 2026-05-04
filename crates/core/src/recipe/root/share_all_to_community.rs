use evento::{Executor, ProjectionAggregator};
use imkitchen_types::recipe_share::AllSharedToCommunity;

impl<E: Executor + Clone> super::Module<E> {
    pub async fn share_all_to_community(
        &self,
        request_by: impl Into<String>,
        owner_name: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let owner_name = owner_name.into();

        let share = self.load_share(&request_by).await?;

        share
            .aggregator()?
            .event(&AllSharedToCommunity { owner_name })
            .requested_by(&request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
