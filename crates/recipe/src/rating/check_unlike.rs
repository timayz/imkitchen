use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::recipe::rating::{LikeUnchecked, UnlikeChecked};

impl<E: Executor + Clone> super::Command<E> {
    pub async fn check_unlike(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        let user_id = user_id.into();
        let rating = self.load(&id, &user_id).await?;
        if !rating.unliked {
            rating
                .aggregator()?
                .event(&UnlikeChecked)
                .metadata(&Metadata::new(&rating.user_id))
                .commit(&self.executor)
                .await?;
        }
        if rating.liked {
            let rating = self.load(id, user_id).await?;
            rating
                .aggregator()?
                .event(&LikeUnchecked)
                .metadata(&Metadata::new(&rating.user_id))
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
