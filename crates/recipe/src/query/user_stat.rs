use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::RecipeUserStat;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

use crate::{Created, Deleted, Imported, MadePrivate, SharedToCommunity};
use sqlx::{SqlitePool, prelude::FromRow};

#[derive(Default, FromRow)]
pub struct UserStatView {
    pub total: u32,
    pub favorite: u32,
    pub shared: u32,
    pub from_community: u32,
}

pub async fn find_user_stat(
    pool: &SqlitePool,
    user_id: impl Into<String>,
) -> anyhow::Result<Option<UserStatView>> {
    let user_id = user_id.into();
    let statement = sea_query::Query::select()
        .columns([
            RecipeUserStat::Total,
            RecipeUserStat::Shared,
            RecipeUserStat::Favorite,
            RecipeUserStat::FromCommunity,
        ])
        .from(RecipeUserStat::Table)
        .and_where(Expr::col(RecipeUserStat::UserId).eq(user_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with(&sql, values)
        .fetch_optional(pool)
        .await?)
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-user-stat-view")
        .handler(handle_created())
        .handler(handle_imported())
        .handler(handle_deleted())
        .handler(handle_shared_to_community())
        .handler(handle_made_private())
}

#[evento::sub_handler]
async fn handle_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.user()?;

    let statement = Query::insert()
        .into_table(RecipeUserStat::Table)
        .columns([RecipeUserStat::UserId, RecipeUserStat::Total])
        .values_panic([user_id.into(), 1.into()])
        .on_conflict(
            OnConflict::column(RecipeUserStat::UserId)
                .value(
                    RecipeUserStat::Total,
                    Expr::col(RecipeUserStat::Total).add(1),
                )
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_imported<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Imported>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.user()?;

    let statement = Query::insert()
        .into_table(RecipeUserStat::Table)
        .columns([RecipeUserStat::UserId, RecipeUserStat::Total])
        .values_panic([user_id.into(), 1.into()])
        .on_conflict(
            OnConflict::column(RecipeUserStat::UserId)
                .value(
                    RecipeUserStat::Total,
                    Expr::col(RecipeUserStat::Total).add(1),
                )
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_deleted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.user()?;

    let statement = Query::insert()
        .into_table(RecipeUserStat::Table)
        .columns([RecipeUserStat::UserId, RecipeUserStat::Total])
        .values_panic([user_id.into(), 1.into()])
        .on_conflict(
            OnConflict::column(RecipeUserStat::UserId)
                .value(
                    RecipeUserStat::Total,
                    Expr::col(RecipeUserStat::Total).sub(1),
                )
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_shared_to_community<E: Executor>(
    context: &Context<'_, E>,
    event: Event<SharedToCommunity>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.user()?;

    let statement = Query::insert()
        .into_table(RecipeUserStat::Table)
        .columns([RecipeUserStat::UserId, RecipeUserStat::Shared])
        .values_panic([user_id.into(), 1.into()])
        .on_conflict(
            OnConflict::column(RecipeUserStat::UserId)
                .value(
                    RecipeUserStat::Shared,
                    Expr::col(RecipeUserStat::Shared).add(1),
                )
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_made_private<E: Executor>(
    context: &Context<'_, E>,
    event: Event<MadePrivate>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.user()?;

    let statement = Query::insert()
        .into_table(RecipeUserStat::Table)
        .columns([RecipeUserStat::UserId, RecipeUserStat::Shared])
        .values_panic([user_id.into(), 1.into()])
        .on_conflict(
            OnConflict::column(RecipeUserStat::UserId)
                .value(
                    RecipeUserStat::Shared,
                    Expr::col(RecipeUserStat::Shared).sub(1),
                )
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
