use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::user::UsernameChanged;
use regex::Regex;
use std::sync::LazyLock;
use validator::Validate;

use crate::root::repository;

static RE_ALPHA_NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_]+$").unwrap());

#[derive(Validate)]
pub struct SetUsernameInput {
    #[validate(length(min = 3, max = 15), regex(path = *RE_ALPHA_NUM, message = "Only letters (A-Z, a-z) and numbers (0-9) are allowed."))]
    pub username: String,
}

impl<E: Executor> super::Command<E> {
    pub async fn set_username(
        &self,
        id: impl Into<String>,
        username: String,
    ) -> imkitchen_shared::Result<()> {
        let input = SetUsernameInput { username };
        input.validate()?;

        let Some(user) =
            repository::find(&self.read_db, repository::FindType::Id(id.into())).await?
        else {
            imkitchen_shared::not_found!("user not found");
        };

        if user.username.is_some() {
            imkitchen_shared::user!("Username has already been set");
        }

        if repository::is_username_exists(&self.read_db, &input.username).await? {
            imkitchen_shared::user!("Username already used");
        }

        repository::update(
            &self.write_db,
            repository::UpdateInput {
                id: user.id.to_owned(),
                username: Some(input.username.to_owned()),
                password: None,
                role: None,
                state: None,
            },
        )
        .await?;

        let Some(user) = self.load(&user.id).await? else {
            imkitchen_shared::server!("user in set_username");
        };

        user.aggregator()?
            .event(&UsernameChanged {
                value: input.username,
            })
            .requested_by(user.id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
