use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use evento::{Executor, LoadResult};
use imkitchen_shared::Metadata;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use validator::Validate;

use crate::reset_password::UserResetPassword;

use super::{ResetRequested, Resetted};

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<UserResetPassword>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }
}

#[derive(Validate)]
pub struct RequestInput {
    #[validate(email)]
    pub email: String,
    pub lang: String,
    pub host: String,
}
impl<E: Executor + Clone> Command<E> {
    pub async fn request(
        &self,
        input: RequestInput,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        input.validate()?;

        evento::create::<UserResetPassword>()
            .data(&ResetRequested {
                email: input.email,
                lang: input.lang,
                host: input.host,
            })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}

#[derive(Validate)]
pub struct ResetInput {
    pub id: String,
    #[validate(length(min = 8, max = 20))]
    pub password: String,
}
impl<E: Executor + Clone> Command<E> {
    pub async fn reset(&self, input: ResetInput) -> imkitchen_shared::Result<Option<()>> {
        input.validate()?;

        let Some(loaded) = self.load(&input.id).await? else {
            return Ok(None);
        };

        let expire_secs: u64 = (OffsetDateTime::now_utc() + time::Duration::minutes(15))
            .unix_timestamp()
            .try_into()?;

        if loaded.event.timestamp > expire_secs {
            imkitchen_shared::bail!("token expired");
        }

        if loaded.item.resetted {
            imkitchen_shared::bail!("has already been reset");
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(input.password.as_bytes(), &salt)?
            .to_string();

        let user_id = loaded.item.user_id.to_owned();

        evento::save_with(loaded)
            .data(&Resetted { password_hash })?
            .metadata(&Metadata::by(user_id))?
            .commit(&self.0)
            .await?;

        Ok(Some(()))
    }
}
