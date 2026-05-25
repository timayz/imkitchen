pub mod thumbnail;
pub mod user;
pub mod user_fts;
pub mod user_stat;

use evento::{
    AggregatorEvent, Executor,
    metadata::RawEvent,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::recipe_user::RecipeUser;
use imkitchen_types::recipe;
use imkitchen_types::recipe_share::{AllMadePrivate, AllSharedToCommunity};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;
use evento::metadata::Event;

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-query")
        .handler(handle_recipe_all())
        .handler(handle_all_shared_to_community())
        .handler(handle_all_made_private())
        .safety_check()
}

#[evento::subscription_all]
async fn handle_recipe_all<E: Executor>(
    context: &Context<'_, E>,
    event: RawEvent<recipe::Created>,
) -> anyhow::Result<()> {
    let (r, w) = context.extract::<(SqlitePool, SqlitePool)>();
    if event.name != recipe::Deleted::event_name() {
        user::load(context.executor, &r, &w, &event.aggregator_id).await?;
        return Ok(());
    }
    let (sql, values) = sea_query::Query::delete()
        .from_table(RecipeUser::Table)
        .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&w).await?;
    Ok(())
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

    sqlx::query_with(&sql, values).execute(&w).await?;

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

    sqlx::query_with(&sql, values).execute(&w).await?;

    Ok(())
}
