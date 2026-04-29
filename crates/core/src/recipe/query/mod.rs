pub mod comment;
pub mod thumbnail;
pub mod user;
pub mod user_fts;
pub mod user_stat;

use evento::{
    AggregatorEvent, Executor,
    metadata::{Event, RawEvent},
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::RecipeUser;
use imkitchen_shared::recipe::{self, comment::Replied};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;
use std::ops::Deref;

#[derive(Clone)]
pub struct Query<E: Executor>(pub imkitchen_shared::State<E>);

impl<E: Executor> Deref for Query<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-query")
        .handler(handle_recipe_all())
        .handler(handle_comment_added())
        .skip::<Replied>()
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
async fn handle_comment_added<E: Executor>(
    context: &Context<'_, E>,
    event: Event<recipe::comment::Added>,
) -> anyhow::Result<()> {
    let (r, w) = context.extract::<(SqlitePool, SqlitePool)>();
    comment::load(
        context.executor,
        &r,
        &w,
        &event.data.recipe_id,
        event.metadata.requested_by()?,
    )
    .await?;
    Ok(())
}
