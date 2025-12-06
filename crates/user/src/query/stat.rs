use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::UserStat;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

use crate::{RegistrationSucceeded, User};

#[derive(Default, FromRow)]
pub struct UserStatRow {
    pub total: u32,
    pub premium: u32,
    pub suspended: u32,
}

impl super::Query {
    pub async fn find_stat(&self, day: u64) -> anyhow::Result<Option<UserStatRow>> {
        let statement = sea_query::Query::select()
            .columns([UserStat::Total, UserStat::Premium, UserStat::Suspended])
            .from(UserStat::Table)
            .and_where(Expr::col(UserStat::Day).eq(day))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        Ok(sqlx::query_as_with::<_, UserStatRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}

pub fn subscribe_stat<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("user-stat")
        .handler(handle_registration_succeeded())
        .handler_check_off()
}

#[evento::handler(User)]
async fn handle_registration_succeeded<E: Executor>(
    context: &evento::Context<'_, E>,
    _event: Event<RegistrationSucceeded>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::insert()
        .into_table(UserStat::Table)
        .columns([UserStat::Day, UserStat::Total])
        .values_panic([0.into(), 1.into()])
        .on_conflict(
            OnConflict::column(UserStat::Day)
                .value(UserStat::Total, Expr::col(UserStat::Total).add(1))
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
