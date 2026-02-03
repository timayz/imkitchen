use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::comment_rating::UnlikeUnchecked;

impl<E: Executor + Clone> super::Command<E> {
    pub async fn uncheck_unlike(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        let user_id = user_id.into();
        let rating = self.load(&id, &user_id).await?;
        if rating.unliked {
            rating
                .aggregator()?
                .event(&UnlikeUnchecked { comment_id: id })
                .requested_by(user_id)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
