use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::rating::Viewed;

impl<E: Executor + Clone> super::Command<E> {
    pub async fn view(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let rating = self.load(id, user_id).await?;
        if !rating.viewed {
            rating
                .aggregator()?
                .event(&Viewed)
                .requested_by(&rating.user_id)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
