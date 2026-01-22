use evento::Executor;
use imkitchen_shared::recipe::comment::Added;
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
        evento::create()
            .event(&Added {
                recipe_id: id.into(),
                message: input.message,
                reply_to: input.reply_to,
            })
            .requested_by(user_id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
