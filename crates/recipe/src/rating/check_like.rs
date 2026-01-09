use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::recipe::rating::LikeChecked;

impl<E: Executor + Clone> super::Command<E> {
    pub async fn check_like(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let rating = self.load(id, user_id).await?;

        if !rating.liked {
            rating
                .aggregator()?
                .event(&LikeChecked)
                .metadata(&Metadata::new(&rating.user_id))
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
