use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use evento::{AggregatorName, Executor, LoadResult, SubscribeBuilder};
use imkitchen_shared::{Event, Metadata};
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::{SqlitePool, prelude::FromRow};
use validator::Validate;

use crate::{
    LoggedIn, MadeAdmin, RegistrationFailed, RegistrationRequested, RegistrationSucceeded, Role,
    User,
    meal_preferences::{self, UserMealPreferences},
};
use imkitchen_db::table::User as UserIden;

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

#[derive(Validate)]
pub struct UpdateMealPreferencesInput {
    #[validate(range(min = 1))]
    pub household_size: u8,
    pub dietary_restrictions: Vec<String>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub cuisine_variety_weight: f32,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct AuthUser {
    pub id: String,
    pub role: String,
}

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(&self, id: impl Into<String>) -> Result<LoadResult<User>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn load_meal_preferences(
        &self,
        id: impl Into<String>,
    ) -> Result<LoadResult<UserMealPreferences>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn get_user_by_id(
        &self,
        id: impl Into<String>,
    ) -> imkitchen_shared::Result<Option<AuthUser>> {
        let id = id.into();

        let statement = Query::select()
            .columns([UserIden::Id, UserIden::Role])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(Expr::value(id.to_owned())))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, AuthUser, _>(&sql, values)
            .fetch_optional(&self.1)
            .await?)
    }

    pub async fn login(
        &self,
        input: LoginInput,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<String> {
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
            imkitchen_shared::bail!("Invalid email or password. Please try again.");
        };

        let user = evento::load::<User, _>(&self.0, &user_id).await?;
        let parsed_hash = PasswordHash::new(&user.item.password_hash)?;
        let argon2 = Argon2::default();

        if argon2
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .is_err()
        {
            imkitchen_shared::bail!("Invalid email or password. Please try again.");
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
    ) -> imkitchen_shared::Result<String> {
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

    pub async fn update_meal_preferences(
        &self,
        input: UpdateMealPreferencesInput,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<()> {
        input.validate()?;

        for restriction in &input.dietary_restrictions {
            if !matches!(
                restriction.as_str(),
                "vegetarian" | "vegan" | "gluten-free" | "dairy-free" | "nut-free" | "low-carb"
            ) {
                imkitchen_shared::bail!(
                    "Please select a valid dietary restriction: vegetarian, vegan, gluten-free, dairy-free, nut-free, or low-carb."
                )
            }
        }

        let Some(user_id) = metadata.trigger_by() else {
            imkitchen_shared::bail!("User not found in metadata");
        };

        let builder = match evento::load::<UserMealPreferences, _>(&self.0, &user_id).await {
            Ok(preferences) => evento::save_with(preferences).data(&meal_preferences::Updated {
                dietary_restrictions: input.dietary_restrictions,
                household_size: input.household_size,
                cuisine_variety_weight: input.cuisine_variety_weight,
            })?,
            Err(evento::ReadError::NotFound) => {
                evento::create_with(user_id).data(&meal_preferences::Updated {
                    dietary_restrictions: input.dietary_restrictions,
                    household_size: input.household_size,
                    cuisine_variety_weight: input.cuisine_variety_weight,
                })?
            }
            Err(e) => return Err(e.into()),
        };

        builder.metadata(&metadata)?.commit(&self.0).await?;

        Ok(())
    }
}

#[evento::handler(User)]
async fn handle_registration_requested<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<RegistrationRequested>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::insert()
        .into_table(UserIden::Table)
        .columns([UserIden::Id, UserIden::Email, UserIden::Role])
        .values_panic([
            event.aggregator_id.to_string().into(),
            event.data.email.to_string().into(),
            Role::User.to_string().into(),
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

#[evento::handler(User)]
async fn handle_made_admin<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MadeAdmin>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(UserIden::Table)
        .values([(UserIden::Role, event.data.role.to_owned().into())])
        .and_where(Expr::col(UserIden::Id).eq(event.aggregator_id.to_owned()))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

pub fn subscribe_command<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("user-command")
        .handler(handle_registration_requested())
        .handler(handle_made_admin())
        .skip::<User, RegistrationSucceeded>()
        .skip::<User, RegistrationFailed>()
        .skip::<User, LoggedIn>()
        .skip::<UserMealPreferences, meal_preferences::Created>()
        .skip::<UserMealPreferences, meal_preferences::Updated>()
}
