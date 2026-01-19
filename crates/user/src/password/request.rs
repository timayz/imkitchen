use evento::Executor;
use imkitchen_shared::user::password::ResetRequested;
use validator::Validate;

use crate::repository::FindType;

#[derive(Validate)]
pub struct RequestInput {
    #[validate(email)]
    pub email: String,
    pub lang: String,
    pub host: String,
}

impl<E: Executor> super::Command<E> {
    pub async fn request(&self, input: RequestInput) -> imkitchen_shared::Result<Option<String>> {
        input.validate()?;

        let Some(user) =
            crate::repository::find(&self.read_db, FindType::Email(input.email.to_owned())).await?
        else {
            return Ok(None);
        };

        let id = evento::create()
            .event(&ResetRequested {
                user_id: user.id,
                email: input.email,
                lang: input.lang,
                host: input.host,
            })
            .commit(&self.executor)
            .await?;

        Ok(Some(id))
    }
}
