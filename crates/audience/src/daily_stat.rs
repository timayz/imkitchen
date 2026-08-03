use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use sea_query::{Alias, Expr, ExprTrait, Func, OnConflict, Order, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;
use time::{UtcDateTime, macros::format_description};

use crate::audience_daily_stat::AudienceDailyStat;
use crate::types::Visited;

#[derive(Default, Debug, FromRow)]
pub struct DayTotal {
    pub day: String,
    pub total: u32,
}

#[derive(Default, Debug, FromRow)]
pub struct BreakdownRow {
    pub label: String,
    pub total: u32,
}

#[derive(Clone, Copy)]
pub enum BreakdownDim {
    Path,
    Device,
    Browser,
    Country,
    Referrer,
}

impl BreakdownDim {
    fn column(self) -> AudienceDailyStat {
        match self {
            Self::Path => AudienceDailyStat::Path,
            Self::Device => AudienceDailyStat::Device,
            Self::Browser => AudienceDailyStat::Browser,
            Self::Country => AudienceDailyStat::Country,
            Self::Referrer => AudienceDailyStat::Referrer,
        }
    }
}

/// Zero-padded `YYYY-MM-DD` from a unix timestamp, so days sort as strings.
pub fn day_string(timestamp: u64) -> anyhow::Result<String> {
    let date = UtcDateTime::from_unix_timestamp(timestamp.try_into()?)?.date();

    Ok(date.format(format_description!("[year]-[month]-[day]"))?)
}

impl<E: Executor> crate::Module<E> {
    /// Sum of visits for `day >= from_day`.
    pub async fn total_since(&self, from_day: &str) -> anyhow::Result<u32> {
        let statement = Query::select()
            .expr(Func::coalesce([
                Expr::from(Func::sum(Expr::col(AudienceDailyStat::Total))),
                Expr::val(0),
            ]))
            .from(AudienceDailyStat::Table)
            .and_where(Expr::col(AudienceDailyStat::Day).gte(from_day))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        Ok(sqlx::query_scalar_with(sqlx::AssertSqlSafe(sql), values)
            .fetch_one(&self.read_db)
            .await?)
    }

    /// Per-day visit totals for `day >= from_day`, most recent first.
    pub async fn per_day(&self, from_day: &str) -> anyhow::Result<Vec<DayTotal>> {
        let statement = Query::select()
            .column(AudienceDailyStat::Day)
            .expr_as(
                Func::sum(Expr::col(AudienceDailyStat::Total)),
                Alias::new("total"),
            )
            .from(AudienceDailyStat::Table)
            .and_where(Expr::col(AudienceDailyStat::Day).gte(from_day))
            .group_by_col(AudienceDailyStat::Day)
            .order_by(AudienceDailyStat::Day, Order::Desc)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        Ok(sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(&self.read_db)
            .await?)
    }

    /// Top values of one dimension for `day >= from_day`, highest first.
    pub async fn breakdown(
        &self,
        dim: BreakdownDim,
        from_day: &str,
        limit: u64,
    ) -> anyhow::Result<Vec<BreakdownRow>> {
        let statement = Query::select()
            .expr_as(Expr::col(dim.column()), Alias::new("label"))
            .expr_as(
                Func::sum(Expr::col(AudienceDailyStat::Total)),
                Alias::new("total"),
            )
            .from(AudienceDailyStat::Table)
            .and_where(Expr::col(AudienceDailyStat::Day).gte(from_day))
            .group_by_col(dim.column())
            .order_by(Alias::new("total"), Order::Desc)
            .limit(limit)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        Ok(sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(&self.read_db)
            .await?)
    }
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("audience-daily-stat").handler(handle_page_visited())
}

#[evento::subscription]
async fn handle_page_visited<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Visited>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let day = day_string(event.timestamp)?;

    let statement = Query::insert()
        .into_table(AudienceDailyStat::Table)
        .columns([
            AudienceDailyStat::Day,
            AudienceDailyStat::Path,
            AudienceDailyStat::Device,
            AudienceDailyStat::Browser,
            AudienceDailyStat::Country,
            AudienceDailyStat::Referrer,
            AudienceDailyStat::Total,
            AudienceDailyStat::CreatedAt,
        ])
        .values([
            day.into(),
            event.data.path.to_owned().into(),
            event.data.device.to_owned().into(),
            event.data.browser.to_owned().into(),
            event.data.country.to_owned().into(),
            event.data.referrer.to_owned().into(),
            1.into(),
            event.timestamp.into(),
        ])?
        .on_conflict(
            OnConflict::columns([
                AudienceDailyStat::Day,
                AudienceDailyStat::Path,
                AudienceDailyStat::Device,
                AudienceDailyStat::Browser,
                AudienceDailyStat::Country,
                AudienceDailyStat::Referrer,
            ])
            .value(
                AudienceDailyStat::Total,
                Expr::col(AudienceDailyStat::Total).add(1),
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
