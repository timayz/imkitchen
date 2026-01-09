use argon2::{Argon2, PasswordHash, PasswordVerifier};
use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::user::{LoggedIn, Logout, State};
use ulid::Ulid;
use validator::Validate;

use crate::root::repository;

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

impl<E: Executor> super::Command<E> {
    pub async fn login(&self, input: LoginInput) -> imkitchen_shared::Result<(String, String)> {
        input.validate()?;

        let Some(user_row) =
            repository::find(&self.read_db, repository::FindType::Email(input.email)).await?
        else {
            imkitchen_shared::user!("Invalid email or password. Please try again.");
        };

        let Some(user) = self.load(&user_row.id).await? else {
            imkitchen_shared::server!("User not found in login");
        };

        let parsed_hash = PasswordHash::new(&user_row.password)?;
        let argon2 = Argon2::default();

        if argon2
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .is_err()
        {
            imkitchen_shared::user!("Invalid email or password. Please try again.");
        }

        if user.state == State::Suspended {
            imkitchen_shared::user!("Account suspended");
        }

        let access_id = Ulid::new().to_string();
        let subscription = self.subscription.load(&user_row.id).await?;

        user.aggregator()?
            .event(&LoggedIn {
                role: user_row.role.0.to_owned(),
                state: user_row.state.0.to_owned(),
                username: user_row.username,
                subscription_expire_at: subscription.expire_at,
                lang: input.lang,
                timezone: input.timezone,
                user_agent: input.user_agent,
                access_id: access_id.to_owned(),
            })
            .metadata(&Metadata::default())
            .commit(&self.executor)
            .await?;

        Ok((user_row.id, access_id))
    }

    pub async fn logout(
        &self,
        id: impl Into<String>,
        access_id: String,
    ) -> imkitchen_shared::Result<String> {
        let Some(user) = self.load(id).await? else {
            imkitchen_shared::not_found!("user in logout");
        };

        user.aggregator()?
            .event(&Logout {
                access_id: access_id.to_owned(),
            })
            .metadata(&Metadata::default())
            .commit(&self.executor)
            .await?;

        Ok(access_id)
    }
}
