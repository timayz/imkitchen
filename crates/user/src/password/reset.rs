use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::user::password::ResetCompleted;
use time::OffsetDateTime;
use validator::Validate;

use crate::repository::{self};

#[derive(Validate)]
pub struct ResetInput {
    pub id: String,
    #[validate(length(min = 8, max = 20))]
    pub password: String,
}

impl<E: Executor> super::Command<E> {
    pub async fn reset(&self, input: ResetInput) -> imkitchen_shared::Result<()> {
        input.validate()?;

        let Some(password) = self.load(&input.id).await? else {
            imkitchen_shared::not_found!("password");
        };
        let now: u64 = OffsetDateTime::now_utc().unix_timestamp().try_into()?;

        if now > password.expire_at {
            imkitchen_shared::user!("token expired");
        }

        if password.completed {
            imkitchen_shared::user!("has already been reset");
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(input.password.as_bytes(), &salt)?
            .to_string();

        repository::update(
            &self.write_db,
            repository::UpdateInput {
                id: password.user_id.to_owned(),
                username: None,
                password: Some(password_hash),
                role: None,
                state: None,
            },
        )
        .await?;

        password
            .aggregator()?
            .event(&ResetCompleted)
            .metadata(&Metadata::new(&password.user_id))
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
