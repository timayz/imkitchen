use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::comment_rating::{LikeChecked, UnlikeUnchecked};

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
                .event(&LikeChecked {
                    comment_id: id.to_owned(),
                })
                .requested_by(&user_id)
                .commit(&self.executor)
                .await?;
        }

        if rating.unliked {
            let rating = self.load(&id, &user_id).await?;
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
