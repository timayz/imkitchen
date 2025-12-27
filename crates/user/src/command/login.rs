use argon2::{Argon2, PasswordHash, PasswordVerifier};
use evento::{Executor, metadata::Metadata};
use sqlx::SqlitePool;
use ulid::Ulid;
use validator::Validate;

use crate::{LoggedIn, Logout, State, command::repository};

#[derive(Validate)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub lang: String,
    pub timezone: String,
    pub user_agent: String,
}

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn login(
        executor: &E,
        read_pool: &SqlitePool,
        input: LoginInput,
    ) -> imkitchen_shared::Result<(String, String)> {
        input.validate()?;

        let Some(user) =
            repository::find(read_pool, repository::FindType::Email(input.email)).await?
        else {
            imkitchen_shared::user!("Invalid email or password. Please try again.");
        };

        let Some(command) = super::load(executor, read_pool, &user.id).await? else {
            imkitchen_shared::server!("User command not found");
        };

        let parsed_hash = PasswordHash::new(&user.password)?;
        let argon2 = Argon2::default();

        if argon2
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .is_err()
        {
            imkitchen_shared::user!("Invalid email or password. Please try again.");
        }

        if command.state == State::Suspended {
            imkitchen_shared::user!("Account suspended");
        }

        let access_id = Ulid::new().to_string();

        command
            .aggregator()
            .event(&LoggedIn {
                lang: input.lang,
                timezone: input.timezone,
                user_agent: input.user_agent,
                access_id: access_id.to_owned(),
            })
            .metadata(&Metadata::default())
            .commit(executor)
            .await?;

        Ok((user.id, access_id))
    }

    pub async fn logout(&self, access_id: String) -> imkitchen_shared::Result<String> {
        self.aggregator()
            .event(&Logout {
                access_id: access_id.to_owned(),
            })
            .metadata(&Metadata::default())
            .commit(self.executor)
            .await?;

        Ok(access_id)
    }
}
