use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use evento::{AggregatorName, Executor, LoadResult, SubscribeBuilder};
use imkitchen_shared::{Event, Metadata};
use regex::Regex;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::{SqlitePool, prelude::FromRow};
use std::sync::LazyLock;
use time::OffsetDateTime;
use ulid::Ulid;
use validator::Validate;

use crate::{
    Activated, LoggedIn, MadeAdmin, RegistrationFailed, RegistrationRequested,
    RegistrationSucceeded, Role, State, Status, Suspended, User,
    meal_preferences::{self, UserMealPreferences},
    reset_password::{self, Resetted, UserResetPassword},
    subscription::{LifePremiumToggled, UserSubscription},
};
use imkitchen_db::table::{User as UserIden, UserLogin};

#[evento::handler(UserSubscription)]
async fn handle_life_premium_toggled<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<LifePremiumToggled>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(UserIden::Table)
        .values([(UserIden::SubscriptionExpireAt, event.data.expire_at.into())])
        .and_where(Expr::col(UserIden::Id).eq(event.aggregator_id.to_owned()))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(UserResetPassword)]
async fn handle_reset_password_resetted<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Resetted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let id = event.metadata.trigger_by()?;
    evento::save::<User>(&id)
        .data(&event.data)?
        .metadata(&event.metadata)?
        .commit(context.executor)
        .await?;

    let statement = Query::delete()
        .from_table(UserLogin::Table)
        .and_where(Expr::col(UserLogin::UserId).eq(id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
