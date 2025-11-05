use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use evento::{AggregatorName, Executor, LoadResult, SubscribeBuilder};
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;
use validator::Validate;

use crate::{
    LoggedIn, Metadata, RegistrationFailed, RegistrationRequested, RegistrationSucceeded, User,
    UserEvent, sql::user::User as UserIden,
};

#[derive(Validate)]
pub struct RegisterInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 20))]
    pub password: String,
}

#[derive(Validate)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 20))]
    pub password: String,
    pub lang: String,
}

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(&self, id: impl Into<String>) -> Result<LoadResult<User>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn login(&self, input: LoginInput, metadata: Metadata) -> anyhow::Result<String> {
        input.validate()?;

        let statement = Query::select()
            .columns([UserIden::Id])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Email).eq(Expr::value(input.email.to_owned())))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        let Some((user_id,)) = sqlx::query_as_with::<_, (String,), _>(&sql, values)
            .fetch_optional(&self.1)
            .await?
        else {
            anyhow::bail!("Invalid email or password. Please try again.");
        };

        let user = evento::load::<User, _>(&self.0, &user_id).await?;
        let parsed_hash = PasswordHash::new(&user.item.password_hash)?;
        let argon2 = Argon2::default();

        if argon2
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .is_err()
        {
            anyhow::bail!("Invalid email or password. Please try again.");
        }

        Ok(evento::save_with(user)
            .data(&LoggedIn { lang: input.lang })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?)
    }

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
        .into_table(UserIden::Table)
        .columns([UserIden::Id, UserIden::Email])
        .values_panic([
            event.aggregator_id.to_string().into(),
            event.data.email.to_string().into(),
        ])
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let Err(e) = sqlx::query_with(&sql, values).execute(&pool).await else {
        evento::save::<User>(&event.aggregator_id)
            .data(&RegistrationSucceeded {
                email: event.data.email,
            })?
            .metadata(&event.metadata)?
            .commit(context.executor)
            .await?;
        return Ok(());
    };

    if !e
        .to_string()
        .contains("UNIQUE constraint failed: user.email")
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
        .skip::<User, LoggedIn>()
}
