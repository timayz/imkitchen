use evento::{Executor, metadata::Metadata};
use validator::Validate;

use crate::{FormSubmitted, Subject};

#[derive(Validate)]
pub struct SubmitFormInput {
    #[validate(email)]
    pub to: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 25))]
    pub name: String,
    pub subject: Subject,
    #[validate(length(min = 1, max = 2000))]
    pub message: String,
}

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn submit_form(
        executor: &E,
        input: SubmitFormInput,
    ) -> imkitchen_shared::Result<String> {
        input.validate()?;

        Ok(evento::create()
            .event(&FormSubmitted {
                to: input.to,
                name: input.name,
                email: input.email,
                subject: input.subject,
                message: input.message,
            })
            .metadata(&Metadata::default())
            .commit(executor)
            .await?)
    }
}
