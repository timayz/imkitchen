//! Followed chefs: the `chef_follower` list and the `follower` count on the
//! chef's `recipe_user_stat` row, both maintained by one subscription so the
//! count can never drift from the list (a replayed event that changes no row
//! changes no count either).

use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::{chef_follower::ChefFollower, recipe_user_stat::RecipeUserStat};
use imkitchen_types::chef_follow::{Followed, Unfollowed};
use sea_query::{
    Alias, Expr, ExprTrait, Func, JoinType, OnConflict, Order, Query, SqliteQueryBuilder,
};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

/// One chef the user follows, newest follow first.
#[derive(Debug, Clone, Default, FromRow)]
pub struct FollowedChefView {
    pub chef_id: String,
    pub chef_name: String,
    pub shared: u32,
    pub follower: u32,
}

impl<E: Executor> crate::recipe::Module<E> {
    pub async fn list_followed_chefs(
        &self,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Vec<FollowedChefView>> {
        let statement = Query::select()
            .columns([
                (ChefFollower::Table, ChefFollower::ChefId),
                (ChefFollower::Table, ChefFollower::ChefName),
            ])
            .expr_as(
                Func::coalesce([
                    Expr::col((RecipeUserStat::Table, RecipeUserStat::Shared)),
                    Expr::val(0),
                ]),
                Alias::new("shared"),
            )
            .expr_as(
                Func::coalesce([
                    Expr::col((RecipeUserStat::Table, RecipeUserStat::Follower)),
                    Expr::val(0),
                ]),
                Alias::new("follower"),
            )
            .from(ChefFollower::Table)
            .join(
                JoinType::LeftJoin,
                RecipeUserStat::Table,
                Expr::col((RecipeUserStat::Table, RecipeUserStat::UserId))
                    .equals((ChefFollower::Table, ChefFollower::ChefId)),
            )
            .and_where(Expr::col((ChefFollower::Table, ChefFollower::UserId)).eq(user_id.into()))
            .order_by((ChefFollower::Table, ChefFollower::CreatedAt), Order::Desc)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(&self.read_db)
            .await?)
    }
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-follower-view")
        .handler(handle_followed())
        .handler(handle_unfollowed())
}

#[evento::subscription]
async fn handle_followed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Followed>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.requested_by()?;

    let statement = Query::insert()
        .into_table(ChefFollower::Table)
        .columns([
            ChefFollower::UserId,
            ChefFollower::ChefId,
            ChefFollower::ChefName,
            ChefFollower::CreatedAt,
        ])
        .values_panic([
            user_id.into(),
            event.data.chef_id.to_owned().into(),
            event.data.chef_name.to_owned().into(),
            event.timestamp.into(),
        ])
        .on_conflict(
            OnConflict::columns([ChefFollower::UserId, ChefFollower::ChefId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    let inserted = sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?
        .rows_affected();

    if inserted == 0 {
        return Ok(());
    }

    let statement = Query::insert()
        .into_table(RecipeUserStat::Table)
        .columns([RecipeUserStat::UserId, RecipeUserStat::Follower])
        .values_panic([event.data.chef_id.to_owned().into(), 1.into()])
        .on_conflict(
            OnConflict::column(RecipeUserStat::UserId)
                .value(
                    RecipeUserStat::Follower,
                    Expr::col(RecipeUserStat::Follower).add(1),
                )
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_unfollowed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Unfollowed>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.requested_by()?;

    let statement = Query::delete()
        .from_table(ChefFollower::Table)
        .and_where(Expr::col(ChefFollower::UserId).eq(user_id))
        .and_where(Expr::col(ChefFollower::ChefId).eq(event.data.chef_id.to_owned()))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    let deleted = sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?
        .rows_affected();

    if deleted == 0 {
        return Ok(());
    }

    let statement = Query::update()
        .table(RecipeUserStat::Table)
        .value(
            RecipeUserStat::Follower,
            // SQLite's scalar MAX: never below zero even if a row went missing.
            Expr::cust("MAX(follower - 1, 0)"),
        )
        .and_where(Expr::col(RecipeUserStat::UserId).eq(event.data.chef_id.to_owned()))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}
