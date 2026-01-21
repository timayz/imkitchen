use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::rating::CommentLikeUnchecked;

impl<E: Executor> super::Command<E> {
    pub async fn uncheck_comment_like(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
        comment_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let user_id = user_id.into();
        let rating = self.load(id, &user_id).await?;
        let comment_id = comment_id.into();

        // @TODO: check comment exist
        // @TODO: skip is already done

        rating
            .aggregator()?
            .event(&CommentLikeUnchecked { comment_id })
            .requested_by(user_id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
