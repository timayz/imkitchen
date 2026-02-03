use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::comment_rating::LikeUnchecked;

impl<E: Executor + Clone> super::Command<E> {
    pub async fn uncheck_like(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        let user_id = user_id.into();
        let rating = self.load(&id, &user_id).await?;

        if rating.liked {
            rating
                .aggregator()?
                .event(&LikeUnchecked { comment_id: id })
                .requested_by(user_id)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
