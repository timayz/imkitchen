use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use evento::{Executor, metadata::Metadata};
use imkitchen_shared::user::Registered;
use sqlx::SqlitePool;
use validator::Validate;

use crate::repository;

#[derive(Validate)]
pub struct RegisterInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 20))]
    pub password: String,
    pub lang: String,
    pub timezone: String,
}

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn register(
        executor: &E,
        read_db: &SqlitePool,
        write_db: &SqlitePool,
        input: RegisterInput,
    ) -> imkitchen_shared::Result<String> {
        input.validate()?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(input.password.as_bytes(), &salt)?
            .to_string();

        if repository::find(read_db, repository::FindType::Email(input.email.to_owned()))
            .await?
            .is_some()
        {
            imkitchen_shared::user!("Email already exists");
        }

        let id = evento::create()
            .event(&Registered {
                email: input.email.to_owned(),
                lang: input.lang,
                timezone: input.timezone,
            })
            .metadata(&Metadata::default())
            .commit(executor)
            .await?;

        repository::create(write_db, id.to_owned(), input.email, password_hash).await?;

        Ok(id)
    }
}
