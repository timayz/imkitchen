use evento::{Executor, metadata::Metadata};
use imkitchen_shared::user::UsernameChanged;
use regex::Regex;
use sqlx::SqlitePool;
use std::sync::LazyLock;
use validator::Validate;

use crate::command::repository;

static RE_ALPHA_NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_]+$").unwrap());

#[derive(Validate)]
pub struct SetUsernameInput {
    #[validate(length(min = 3, max = 15), regex(path = *RE_ALPHA_NUM, message = "Only letters (A-Z, a-z) and numbers (0-9) are allowed."))]
    pub username: String,
}

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn set_username(
        &self,
        read_db: &SqlitePool,
        write_db: &SqlitePool,
        username: String,
    ) -> imkitchen_shared::Result<()> {
        let input = SetUsernameInput { username };
        input.validate()?;

        let Some(user) = repository::find(
            read_db,
            repository::FindType::Id(self.aggregator_id.to_owned()),
        )
        .await?
        else {
            imkitchen_shared::not_found!("user not found");
        };

        if user.username.is_some() {
            imkitchen_shared::user!("Username has already been set");
        }

        if repository::is_username_exists(read_db, &input.username).await? {
            imkitchen_shared::user!("Username already used");
        }

        repository::update(
            write_db,
            repository::UpdateInput {
                id: user.id.to_owned(),
                username: Some(input.username.to_owned()),
                password: None,
                role: None,
                state: None,
            },
        )
        .await?;

        self.aggregator()
            .event(&UsernameChanged {
                value: input.username,
            })
            .metadata(&Metadata::new(user.id))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
