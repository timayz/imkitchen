use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::RecipeUserStat;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

use crate::{Created, Deleted, Imported, MadePrivate, Recipe, SharedToCommunity};

pub fn subscribe_user_stat<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("recipe-user-stat")
        .handler(handle_created())
        .handler(handle_imported())
        .handler(handle_deleted())
        .handler(handle_shared_to_community())
        .handler(handle_made_private())
        .handler_check_off()
}

#[evento::handler(Recipe)]
async fn handle_created<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.trigger_by()?;

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

#[evento::handler(Recipe)]
async fn handle_imported<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Imported>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.trigger_by()?;

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

#[evento::handler(Recipe)]
async fn handle_deleted<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.trigger_by()?;

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

#[evento::handler(Recipe)]
async fn handle_shared_to_community<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<SharedToCommunity>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.trigger_by()?;

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

#[evento::handler(Recipe)]
async fn handle_made_private<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MadePrivate>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.trigger_by()?;

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
