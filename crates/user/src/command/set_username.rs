use evento::Executor;
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

        repository::update(
            write_db,
            repository::UpdateInput {
                id: user.id,
                username: Some(input.username),
                password: None,
                role: None,
                state: None,
            },
        )
        .await?;

        Ok(())
    }
}
