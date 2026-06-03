pub mod thumbnail;
pub mod user;
pub mod user_fts;
pub mod user_stat;

use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::recipe_user::RecipeUser;
use imkitchen_types::recipe_share::{AllMadePrivate, AllSharedToCommunity};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-query-share")
        .handler(handle_all_shared_to_community())
        .handler(handle_all_made_private())
        .safety_check()
}

#[evento::subscription]
async fn handle_all_shared_to_community<E: Executor>(
    context: &Context<'_, E>,
    event: Event<AllSharedToCommunity>,
) -> anyhow::Result<()> {
    let (_, w) = context.extract::<(SqlitePool, SqlitePool)>();
    let user_id = event.metadata.requested_by()?;

    let (sql, values) = sea_query::Query::update()
        .table(RecipeUser::Table)
        .value(RecipeUser::IsShared, true)
        .value(RecipeUser::OwnerName, &event.data.owner_name)
        .and_where(Expr::col(RecipeUser::OwnerId).eq(&user_id))
        .and_where(Expr::col(RecipeUser::IsShared).eq(false))
        .and_where(Expr::col(RecipeUser::Name).not_equals(""))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&w)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_all_made_private<E: Executor>(
    context: &Context<'_, E>,
    event: Event<AllMadePrivate>,
) -> anyhow::Result<()> {
    let (_, w) = context.extract::<(SqlitePool, SqlitePool)>();
    let user_id = event.metadata.requested_by()?;

    let (sql, values) = sea_query::Query::update()
        .table(RecipeUser::Table)
        .value(RecipeUser::IsShared, false)
        .and_where(Expr::col(RecipeUser::OwnerId).eq(&user_id))
        .and_where(Expr::col(RecipeUser::IsShared).eq(true))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&w)
        .await?;

    Ok(())
}
