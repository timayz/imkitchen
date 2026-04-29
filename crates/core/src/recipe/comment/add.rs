use evento::Executor;
use imkitchen_shared::recipe::comment::Added;
use validator::Validate;

#[derive(Validate)]
pub struct AddCommentInput {
    #[validate(length(min = 3, max = 2000))]
    pub body: String,
    pub owner_name: String,
}

impl<E: Executor> super::Command<E> {
    pub async fn add(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
        input: AddCommentInput,
    ) -> imkitchen_shared::Result<()> {
        input.validate()?;

        let recipe_id = id.into();
        let user_id = user_id.into();
        evento::AggregatorBuilder::ids(vec![recipe_id.to_owned(), user_id.to_owned()])
            .event(&Added {
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
