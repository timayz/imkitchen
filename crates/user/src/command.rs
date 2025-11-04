use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use evento::Executor;
use validator::Validate;

use crate::{Metadata, RegisterRequested, User};

#[derive(Validate)]
pub struct RegisterInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 20))]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub confirm_password: String,
}

pub struct Command<E: Executor + Clone>(E);

impl<E: Executor + Clone> Command<E> {
    pub async fn register(
        &self,
        input: RegisterInput,
        metadata: Metadata,
    ) -> anyhow::Result<String> {
        input.validate()?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(input.password.as_bytes(), &salt)?
            .to_string();

        Ok(evento::create::<User>()
            .data(&RegisterRequested {
                email: input.email,
                password_hash,
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?)
    }
}
