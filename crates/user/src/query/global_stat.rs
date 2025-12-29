use evento::{
    Action, Executor, Projection, Snapshot, SubscriptionBuilder,
    cursor::{Args, CursorInt, ReadResult},
    metadata::Event,
    sql::Reader,
};
use imkitchen_db::table::UserGlobalStat;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};
use time::UtcDateTime;

use crate::{Activated, Registered, Suspended, subscription::LifePremiumToggled};

static GLOBAL_TIMESTAMP: u64 = 949115824;

#[derive(Default, FromRow, Clone)]
pub struct GlobalStatView {
    pub month: String,
    pub total: u32,
    pub premium: u32,
    pub suspended: u32,
    pub created_at: i64,
}

impl GlobalStatView {
    pub fn total_percent(&self, prev: &Self) -> i64 {
        -Self::percent_change(prev.total.into(), self.total.into())
    }

    pub fn suspended_percent(&self) -> i64 {
        -Self::percent_change(self.total.into(), self.suspended.into())
    }

    pub fn premium_percent(&self, prev: &Self) -> i64 {
        -Self::percent_change(prev.premium.into(), self.premium.into())
    }

    fn percent_change(prev: u64, current: u64) -> i64 {
        if prev == 0 {
            0
        } else {
            ((current as f64 - prev as f64) / prev as f64 * 100.0).round() as i64
        }
    }
}

impl Snapshot for GlobalStatView {}

impl evento::cursor::Cursor for GlobalStatView {
    type T = CursorInt;

    fn serialize(&self) -> Self::T {
        Self::T {
            i: self.month.to_owned(),
            v: self.created_at,
        }
    }
}

impl evento::sql::Bind for GlobalStatView {
    type T = UserGlobalStat;
    type I = [Self::T; 2];
    type V = [Expr; 2];
    type Cursor = Self;

    fn columns() -> Self::I {
        [UserGlobalStat::CreatedAt, UserGlobalStat::Month]
    }

    fn values(
        cursor: <<Self as evento::sql::Bind>::Cursor as evento::cursor::Cursor>::T,
    ) -> Self::V {
        [cursor.v.into(), cursor.i.into()]
    }
}

pub struct FilterQuery {
    pub args: Args,
}

pub async fn find_global(pool: &SqlitePool) -> anyhow::Result<Option<GlobalStatView>> {
    let month = to_month_string(GLOBAL_TIMESTAMP)?;
    let statement = sea_query::Query::select()
        .columns([
            UserGlobalStat::Month,
            UserGlobalStat::Total,
            UserGlobalStat::Premium,
            UserGlobalStat::Suspended,
            UserGlobalStat::CreatedAt,
        ])
        .from(UserGlobalStat::Table)
        .and_where(Expr::col(UserGlobalStat::Month).eq(month))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    Ok(sqlx::query_as_with(&sql, values)
        .fetch_optional(pool)
        .await?)
}

pub async fn filter(
    pool: &SqlitePool,
    input: FilterQuery,
) -> anyhow::Result<ReadResult<GlobalStatView>> {
    let statement = sea_query::Query::select()
        .columns([
            UserGlobalStat::Month,
            UserGlobalStat::Total,
            UserGlobalStat::Premium,
            UserGlobalStat::Suspended,
            UserGlobalStat::CreatedAt,
        ])
        .from(UserGlobalStat::Table)
        .to_owned();

    Reader::new(statement).args(input.args).execute(pool).await
}

async fn update_total(pool: &SqlitePool, timestamp: u64) -> anyhow::Result<()> {
    let month = to_month_string(timestamp)?;
    let statement = Query::insert()
        .into_table(UserGlobalStat::Table)
        .columns([
            UserGlobalStat::Month,
            UserGlobalStat::Total,
            UserGlobalStat::CreatedAt,
        ])
        .values([month.into(), 1.into(), timestamp.into()])?
        .on_conflict(
            OnConflict::column(UserGlobalStat::Month)
                .value(
                    UserGlobalStat::Total,
                    Expr::col(UserGlobalStat::Total).add(1),
                )
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}

async fn update_suspend(pool: &SqlitePool, timestamp: u64, add: bool) -> anyhow::Result<()> {
    let month = to_month_string(timestamp)?;
    let (default_value, expr) = if add {
        (1, Expr::col(UserGlobalStat::Suspended).add(1))
    } else {
        (0, Expr::col(UserGlobalStat::Suspended).sub(1))
    };
    let statement = Query::insert()
        .into_table(UserGlobalStat::Table)
        .columns([
            UserGlobalStat::Month,
            UserGlobalStat::Suspended,
            UserGlobalStat::CreatedAt,
        ])
        .values([month.into(), default_value.into(), timestamp.into()])?
        .on_conflict(
            OnConflict::column(UserGlobalStat::Month)
                .value(UserGlobalStat::Suspended, expr)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}

async fn update_premium(pool: &SqlitePool, timestamp: u64, add: bool) -> anyhow::Result<()> {
    let month = to_month_string(timestamp)?;
    let (default_value, expr) = if add {
        (1, Expr::col(UserGlobalStat::Premium).add(1))
    } else {
        (0, Expr::col(UserGlobalStat::Premium).sub(1))
    };
    let statement = Query::insert()
        .into_table(UserGlobalStat::Table)
        .columns([
            UserGlobalStat::Month,
            UserGlobalStat::Premium,
            UserGlobalStat::CreatedAt,
        ])
        .values([month.into(), default_value.into(), timestamp.into()])?
        .on_conflict(
            OnConflict::column(UserGlobalStat::Month)
                .value(UserGlobalStat::Premium, expr)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}

pub fn create_projection<E: Executor>() -> Projection<GlobalStatView, E> {
    Projection::new("user-global-stat-view")
        .handler(handle_susended())
        .handler(handle_activated())
        .handler(handle_life_premium_toggled())
        .handler(handle_registered())
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<GlobalStatView, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::handler]
async fn handle_registered<E: Executor>(
    event: Event<Registered>,
    action: Action<'_, GlobalStatView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(_data) => {}
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();

            update_total(&pool, GLOBAL_TIMESTAMP).await?;
            update_total(&pool, event.timestamp).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_activated<E: Executor>(
    event: Event<Activated>,
    action: Action<'_, GlobalStatView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(_data) => {}
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();

            update_suspend(&pool, GLOBAL_TIMESTAMP, false).await?;
            update_suspend(&pool, event.timestamp, false).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_susended<E: Executor>(
    event: Event<Suspended>,
    action: Action<'_, GlobalStatView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(_data) => {}
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();

            update_suspend(&pool, GLOBAL_TIMESTAMP, true).await?;
            update_suspend(&pool, event.timestamp, true).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_life_premium_toggled<E: Executor>(
    event: Event<LifePremiumToggled>,
    action: Action<'_, GlobalStatView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(_data) => {}
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            let is_premim = event.data.expire_at > event.timestamp;

            update_premium(&pool, GLOBAL_TIMESTAMP, is_premim).await?;
            update_premium(&pool, event.timestamp, is_premim).await?;
        }
    };

    Ok(())
}

fn to_month_string(timestamp: u64) -> anyhow::Result<String> {
    let date = UtcDateTime::from_unix_timestamp(timestamp.try_into()?)?;

    Ok(format!("{}-{}", date.year(), date.month()))
}
