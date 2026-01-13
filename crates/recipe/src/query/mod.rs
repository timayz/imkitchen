pub mod user;
pub mod user_stat;

use std::ops::Deref;

use evento::{
    AggregatorEvent, Executor, SkipEventData,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::RecipeUser;
use imkitchen_shared::recipe::{Created, Deleted};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct Query<E: Executor>(pub imkitchen_shared::State<E>);

impl<E: Executor> Deref for Query<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn query_subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-query")
        .handler(handle_recipe_all())
        .safety_check()
}

#[evento::sub_all_handler]
async fn handle_recipe_all<E: Executor>(
    context: &Context<'_, E>,
    event: SkipEventData<Created>,
) -> anyhow::Result<()> {
    let (r, w) = context.extract::<(SqlitePool, SqlitePool)>();
    if event.name != Deleted::event_name() {
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
