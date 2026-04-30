use evento::{Executor, ProjectionAggregator};
use imkitchen_types::comment::Replied;
use validator::Validate;

#[derive(Validate)]
pub struct ReplyCommentInput {
    pub comment_id: String,
    #[validate(length(min = 3, max = 2000))]
    pub body: String,
    pub owner_name: String,
}

impl<E: Executor> super::Module<E> {
    pub async fn reply(
        &self,
        recipe_id: impl Into<String>,
        user_id: impl Into<String>,
        input: ReplyCommentInput,
    ) -> crate::Result<()> {
        input.validate()?;

        let recipe_id = recipe_id.into();

        let Some(comment) = self.load_from(&input.comment_id).await? else {
            crate::not_found!("comment");
        };

        comment
            .aggregator()?
            .event(&Replied {
                recipe_id,
                body: input.body,
                owner_name: input.owner_name,
            })
            .requested_by(user_id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
