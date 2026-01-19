use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::rating::{LikeChecked, UnlikeUnchecked};

impl<E: Executor + Clone> super::Command<E> {
    pub async fn check_like(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        let user_id = user_id.into();
        let rating = self.load(&id, &user_id).await?;

        if !rating.liked {
            rating
                .aggregator()?
                .event(&LikeChecked)
                .requested_by(&rating.user_id)
                .commit(&self.executor)
                .await?;
        }

        if rating.unliked {
            let rating = self.load(id, user_id).await?;
            rating
                .aggregator()?
                .event(&UnlikeUnchecked)
                .requested_by(&rating.user_id)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
