use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::recipe::rating::CommentUnlikeUnchecked;

impl<E: Executor> super::Command<E> {
    pub async fn uncheck_comment_unlike(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
        comment_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let rating = self.load(id, user_id).await?;
        let comment_id = comment_id.into();

        // @TODO: check comment exist
        // @TODO: skip if already done

        rating
            .aggregator()?
            .event(&CommentUnlikeUnchecked { comment_id })
            .metadata(&Metadata::new(&rating.user_id))
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
