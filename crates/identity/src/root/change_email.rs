use crate::types::user::EmailChanged;
use evento::{Executor, ProjectionAggregate};
use validator::Validate;

use crate::root::repository;

#[derive(Validate)]
pub struct ChangeEmailInput {
    #[validate(email)]
    pub email: String,
}

impl<E: Executor> super::Module<E> {
    pub async fn change_email(
        &self,
        id: impl Into<String>,
        email: String,
        request_by: impl Into<String>,
    ) -> imkitchen_core::Result<()> {
        let input = ChangeEmailInput { email };
        input.validate()?;

        let id = id.into();

        let Some(user) =
            repository::find(&self.read_db, repository::FindType::Id(id.to_owned())).await?
        else {
            imkitchen_core::not_found!("user not found");
        };

        if user.email.eq_ignore_ascii_case(&input.email) {
            return Ok(());
        }

        if let Some(existing) = repository::find(
            &self.read_db,
            repository::FindType::Email(input.email.to_owned()),
        )
        .await?
            && existing.id != user.id
        {
            imkitchen_core::user!("Email already exists");
        }

        repository::update(
            &self.write_db,
            repository::UpdateInput {
                id: user.id.to_owned(),
                email: Some(input.email.to_owned()),
                username: None,
                password: None,
                role: None,
                state: None,
            },
        )
        .await?;

        let Some(user) = self.load(&user.id).await? else {
            imkitchen_core::server!("user in change_email");
        };

        user.write()?
            .event(&EmailChanged { value: input.email })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
