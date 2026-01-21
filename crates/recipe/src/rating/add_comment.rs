use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::rating::CommentAdded;
use ulid::Ulid;
use validator::Validate;

#[derive(Validate)]
pub struct AddCommentInput {
    pub message: String,
    pub reply_to: Option<String>,
}

impl<E: Executor> super::Command<E> {
    pub async fn add_comment(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
        input: AddCommentInput,
    ) -> imkitchen_shared::Result<()> {
        //@TODO: check spam
        let user_id = user_id.into();
        let rating = self.load(id, &user_id).await?;
        rating
            .aggregator()?
            .event(&CommentAdded {
                id: Ulid::new().to_string(),
                message: input.message,
                reply_to: input.reply_to,
            })
            .requested_by(user_id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
