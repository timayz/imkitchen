use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::recipe::rating::UnlikeUnchecked;

impl<E: Executor + Clone> super::Command<E> {
    pub async fn uncheck_unlike(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let rating = self.load(id, user_id).await?;
        if rating.unliked {
            rating
                .aggregator()?
                .event(&UnlikeUnchecked)
                .metadata(&Metadata::new(&rating.user_id))
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
