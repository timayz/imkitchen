use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    Activated, LoggedIn, MadeAdmin, RegistrationFailed, RegistrationRequested,
    RegistrationSucceeded, Role, Suspended, User,
    meal_preferences::{self, UserMealPreferences},
    subscription::{self, LifePremiumToggled, UserSubscription},
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

#[derive(Default, Debug, Deserialize, FromRow, Clone)]
pub struct AuthUser {
    pub id: String,
    pub role: String,
    pub subscription_end_at: i64,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin.to_string()
    }

    pub fn is_premium(&self) -> bool {
        let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return false;
        };

        self.subscription_end_at as u64 > now.as_secs()
    }
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

    pub async fn load_meal_preferences_optional(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<UserMealPreferences>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }

    pub async fn load_subscription(
        &self,
        id: impl Into<String>,
    ) -> Result<LoadResult<UserSubscription>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn load_subscription_optional(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<UserSubscription>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }

    pub async fn get_user_by_id(
        &self,
        id: impl Into<String>,
    ) -> imkitchen_shared::Result<Option<AuthUser>> {
        let id = id.into();

        let statement = Query::select()
            .columns([UserIden::Id, UserIden::Role, UserIden::SubscriptionEndAt])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(Expr::value(id.to_owned())))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, AuthUser, _>(&sql, values)
            .fetch_optional(&self.1)
            .await?)
    }

    pub async fn get_user_by_email(
        &self,
        email: impl Into<String>,
    ) -> imkitchen_shared::Result<Option<AuthUser>> {
        let email = email.into();

        let statement = Query::select()
            .columns([UserIden::Id, UserIden::Role, UserIden::SubscriptionEndAt])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Email).eq(Expr::value(email.to_owned())))
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

        let Some(user) = self.get_user_by_email(input.email).await? else {
            imkitchen_shared::bail!("Invalid email or password. Please try again.");
        };

        let user = self.load(&user.id).await?;
        let parsed_hash = PasswordHash::new(&user.item.password_hash)?;
        let argon2 = Argon2::default();

        if argon2
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .is_err()
        {
            imkitchen_shared::bail!("Invalid email or password. Please try again.");
        }

        if user.item.role == Role::Suspend {
            imkitchen_shared::bail!("Account suspended");
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

        let user_id = metadata.trigger_by()?;

        evento::save::<UserMealPreferences>(user_id)
            .data(&meal_preferences::Updated {
                dietary_restrictions: input.dietary_restrictions,
                household_size: input.household_size,
                cuisine_variety_weight: input.cuisine_variety_weight,
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn suspend(
        &self,
        id: impl Into<String>,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<()> {
        let user = self.load(id).await?;

        if user.item.role == Role::Suspend {
            return Ok(());
        }

        evento::save_with(user)
            .data(&Suspended {
                role: Role::Suspend.to_string(),
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn activate(
        &self,
        id: impl Into<String>,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<()> {
        let user = self.load(id).await?;

        if user.item.role == Role::User {
            return Ok(());
        }

        evento::save_with(user)
            .data(&Activated {
                role: Role::User.to_string(),
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn made_admin(
        &self,
        id: impl Into<String>,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<()> {
        let user = self.load(id).await?;

        if user.item.role == Role::Admin {
            return Ok(());
        }

        evento::save_with(user)
            .data(&MadeAdmin {
                role: Role::Admin.to_string(),
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn toggle_life_premium(
        &self,
        id: impl Into<String>,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        let expire_at = (SystemTime::now() + Duration::from_secs(10 * 12 * 30 * 24 * 60 * 60))
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let builder = match self.load_subscription_optional(&id).await? {
            Some(subscription) => {
                let expire_at = if subscription.item.expired {
                    expire_at
                } else {
                    0
                };

                evento::save_with(subscription)
                    .data(&subscription::LifePremiumToggled { expire_at })?
            }
            _ => evento::create_with(id).data(&subscription::LifePremiumToggled { expire_at })?,
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
        .columns([
            UserIden::Id,
            UserIden::Email,
            UserIden::Role,
            UserIden::CreatedAt,
        ])
        .values_panic([
            event.aggregator_id.to_string().into(),
            event.data.email.to_string().into(),
            Role::User.to_string().into(),
            event.timestamp.into(),
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

#[evento::handler(User)]
async fn handle_activated<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Activated>,
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

#[evento::handler(User)]
async fn handle_suspended<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Suspended>,
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

#[evento::handler(UserSubscription)]
async fn handle_life_premium_toggled<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<LifePremiumToggled>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(UserIden::Table)
        .values([(UserIden::SubscriptionEndAt, event.data.expire_at.into())])
        .and_where(Expr::col(UserIden::Id).eq(event.aggregator_id.to_owned()))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

pub fn subscribe_command<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("user-command")
        .handler(handle_registration_requested())
        .handler(handle_activated())
        .handler(handle_suspended())
        .handler(handle_made_admin())
        .handler(handle_life_premium_toggled())
        .skip::<User, RegistrationSucceeded>()
        .skip::<User, RegistrationFailed>()
        .skip::<User, LoggedIn>()
        .skip::<UserMealPreferences, meal_preferences::Created>()
        .skip::<UserMealPreferences, meal_preferences::Updated>()
}
