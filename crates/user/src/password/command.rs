use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use evento::{
    Executor, Projection, Snapshot,
    metadata::{Event, Metadata},
};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use validator::Validate;

use crate::{
    password::{Password, ResetCompleted, ResetRequested},
    repository::{self, FindType},
};

#[evento::command]
pub struct Command {
    pub user_id: String,
    pub completed: bool,
    pub expire_at: u64,
}

#[derive(Validate)]
pub struct RequestInput {
    #[validate(email)]
    pub email: String,
    pub lang: String,
    pub host: String,
}
impl<'a, E: Executor + Clone> Command<'a, E> {
    pub async fn request(
        executor: &'a E,
        pool: &'a SqlitePool,
        input: RequestInput,
    ) -> imkitchen_shared::Result<Option<String>> {
        input.validate()?;

        let Some(user) =
            crate::repository::find(pool, FindType::Email(input.email.to_owned())).await?
        else {
            return Ok(None);
        };

        let id = evento::create()
            .event(&ResetRequested {
                user_id: user.id,
                email: input.email,
                lang: input.lang,
                host: input.host,
            })
            .metadata(&Metadata::default())
            .commit(executor)
            .await?;

        Ok(Some(id))
    }
}

#[derive(Validate)]
pub struct ResetInput {
    #[validate(length(min = 8, max = 20))]
    pub password: String,
}

impl<'a, E: Executor + Clone> Command<'a, E> {
    pub async fn reset(
        &self,
        pool: &SqlitePool,
        input: ResetInput,
    ) -> imkitchen_shared::Result<Option<()>> {
        input.validate()?;

        let now: u64 = OffsetDateTime::now_utc().unix_timestamp().try_into()?;

        if now > self.expire_at {
            imkitchen_shared::user!("token expired");
        }

        if self.completed {
            imkitchen_shared::user!("has already been reset");
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(input.password.as_bytes(), &salt)?
            .to_string();

        repository::update(
            pool,
            repository::UpdateInput {
                id: self.user_id.to_owned(),
                username: None,
                password: Some(password_hash),
                role: None,
                state: None,
            },
        )
        .await?;

        self.aggregator()
            .event(&ResetCompleted)
            .metadata(&Metadata::new(&self.user_id))
            .commit(self.executor)
            .await?;

        Ok(Some(()))
    }
}

fn create_projection(id: impl Into<String>) -> Projection<CommandData> {
    Projection::new::<Password>(id)
        .handler(handle_reset_requested())
        .handler(handle_reset_completed())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    id: impl Into<String>,
) -> Result<Option<Command<'a, E>>, anyhow::Error> {
    let id = id.into();

    let Some(data) = create_projection(&id).execute(executor).await? else {
        return Ok(None);
    };

    Ok(Some(Command::new(
        id,
        data.get_cursor_version()?,
        data,
        executor,
    )))
}

impl evento::Snapshot for CommandData {}

#[evento::handler]
async fn handle_reset_requested(
    event: Event<ResetRequested>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.user_id = event.data.user_id.to_owned();
    data.completed = false;
    data.expire_at = (OffsetDateTime::from_unix_timestamp(event.timestamp.try_into()?)?
        + time::Duration::minutes(15))
    .unix_timestamp()
    .try_into()?;

    Ok(())
}

#[evento::handler]
async fn handle_reset_completed(
    _event: Event<ResetCompleted>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.completed = true;

    Ok(())
}
