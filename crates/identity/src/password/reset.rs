use crate::types::password::ResetCompleted;
use argon2::{Argon2, PasswordHasher};
use evento::{Executor, ProjectionAggregate};
use time::OffsetDateTime;
use validator::Validate;

use crate::repository::{self};

use super::Password;

#[derive(Validate)]
pub struct ResetInput {
    pub id: String,
    #[validate(length(min = 8, max = 20))]
    pub password: String,
}

impl<E: Executor> super::Module<E> {
    /// Fails exactly as `reset` would for an unknown, expired or already
    /// used link, so a client can tell before showing the form.
    pub async fn check(&self, id: &str) -> imkitchen_core::Result<()> {
        self.load_usable(id).await?;

        Ok(())
    }

    pub async fn reset(&self, input: ResetInput) -> imkitchen_core::Result<()> {
        input.validate()?;

        let password = self.load_usable(&input.id).await?;

        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(input.password.as_bytes())?.to_string();

        repository::update(
            &self.write_db,
            repository::UpdateInput {
                id: password.user_id.to_owned(),
                email: None,
                username: None,
                password: Some(password_hash),
                role: None,
                state: None,
            },
        )
        .await?;

        password
            .write()?
            .event(&ResetCompleted)
            .requested_by(&password.user_id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    async fn load_usable(&self, id: &str) -> imkitchen_core::Result<Password> {
        let Some(password) = self.load(id).await? else {
            imkitchen_core::not_found!("password");
        };
        let now: u64 = OffsetDateTime::now_utc().unix_timestamp().try_into()?;

        if now > password.expire_at {
            imkitchen_core::user!("token expired");
        }

        if password.completed {
            imkitchen_core::user!("has already been reset");
        }

        Ok(password)
    }
}
