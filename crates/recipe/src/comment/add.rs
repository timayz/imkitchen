use evento::Executor;
use imkitchen_shared::recipe::comment::Added;
use validator::Validate;

#[derive(Validate)]
pub struct AddCommentInput {
    pub body: String,
    pub owner_name: Option<String>,
    pub reply_to: Option<String>,
}

impl<E: Executor> super::Command<E> {
    pub async fn add_comment(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
        input: AddCommentInput,
    ) -> imkitchen_shared::Result<()> {
        let recipe_id = id.into();
        let user_id = user_id.into();
        evento::AggregatorBuilder::ids(vec![
            recipe_id.to_owned(),
            user_id.to_owned(),
            input.reply_to.to_owned().unwrap_or_default(),
        ])
        .event(&Added {
            recipe_id,
            body: input.body,
            reply_to: input.reply_to,
            owner_name: input.owner_name,
        })
        .requested_by(user_id)
        .commit(&self.executor)
        .await?;

        Ok(())
    }
}
