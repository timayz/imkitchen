use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use sea_query::{Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use validator::Validate;

use crate::{
    Metadata, RegistrationFailed, RegistrationRequested, RegistrationSucceeded, User, UserEvent,
    sql::user_emails::UserEmail,
};

#[derive(Validate)]
pub struct RegisterInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 20))]
    pub password: String,
}

pub struct Command<E: Executor + Clone>(pub E);

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
            .data(&RegistrationRequested {
                email: input.email,
                password_hash,
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?)
    }
}

#[evento::handler(User)]
async fn handle_registration_requested<E: Executor>(
    context: &evento::Context<'_, E>,
    event: UserEvent<RegistrationRequested>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::insert()
        .into_table(UserEmail::Table)
        .columns([UserEmail::Email])
        .values_panic([event.data.email.to_string().into()])
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let Err(e) = sqlx::query_with(&sql, values).execute(&pool).await else {
        evento::save::<User>(&event.aggregator_id)
            .data(&RegistrationSucceeded {
                email: event.data.email,
                password_hash: event.data.password_hash,
            })?
            .metadata(&event.metadata)?
            .commit(context.executor)
            .await?;
        return Ok(());
    };

    if !e
        .to_string()
        .contains("UNIQUE constraint failed: user_email.email")
    {
        return Err(e.into());
    }

    evento::save::<User>(&event.aggregator_id)
        .data(&RegistrationFailed {
            reason: "Email already exists".to_owned(),
        })?
        .metadata(&event.metadata)?
        .commit(context.executor)
        .await?;

    Ok(())
}

pub fn subscribe_command<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("user-command")
        .handler(handle_registration_requested())
        .skip::<User, RegistrationFailed>()
        .skip::<User, RegistrationSucceeded>()
}
