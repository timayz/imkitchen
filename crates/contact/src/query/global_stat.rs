use evento::{Action, Executor, Projection, Snapshot, SubscriptionBuilder, metadata::Event};
use imkitchen_db::table::ContactGlobalStat;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};
use time::UtcDateTime;

use crate::{FormSubmitted, MarkedReadAndReply, Resolved};

static GLOBAL_TIMESTAMP: u64 = 949115824;

#[derive(Default, Debug, FromRow)]
pub struct GlobalStatView {
    pub day: String,
    pub total: u32,
    pub unread: u32,
    pub today: u32,
    pub avg_response_time: u32,
}

impl Snapshot for GlobalStatView {}

pub async fn find_global(pool: &SqlitePool) -> anyhow::Result<Option<GlobalStatView>> {
    find(pool, GLOBAL_TIMESTAMP).await
}

pub async fn find(pool: &SqlitePool, day: u64) -> anyhow::Result<Option<GlobalStatView>> {
    let day = to_day_string(day)?;
    let statement = sea_query::Query::select()
        .columns([
            ContactGlobalStat::Day,
            ContactGlobalStat::Total,
            ContactGlobalStat::Today,
            ContactGlobalStat::Unread,
            ContactGlobalStat::AvgResponseTime,
        ])
        .from(ContactGlobalStat::Table)
        .and_where(Expr::col(ContactGlobalStat::Day).eq(day))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    Ok(sqlx::query_as_with(&sql, values)
        .fetch_optional(pool)
        .await?)
}

pub fn create_projection<E: Executor>() -> Projection<GlobalStatView, E> {
    Projection::new("contact-global-stat-view")
        .handler(handle_contact_form_submitted())
        .handler(handle_contact_marked_read_and_reply())
        .handler(handle_contact_resolved())
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<GlobalStatView, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::handler]
async fn handle_contact_form_submitted<E: Executor>(
    event: Event<FormSubmitted>,
    action: Action<'_, GlobalStatView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(_data) => {}
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            update_submitted(&pool, GLOBAL_TIMESTAMP).await?;
            update_submitted(&pool, event.timestamp).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_contact_marked_read_and_reply<E: Executor>(
    event: Event<MarkedReadAndReply>,
    action: Action<'_, GlobalStatView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(_data) => {}
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let Some(contact) = super::admin::find(&pool, &event.aggregator_id).await? else {
                return Ok(());
            };

            if !contact.is_unread() {
                return Ok(());
            }
            update(&pool, ContactGlobalStat::Unread, GLOBAL_TIMESTAMP, false).await?;
            update(&pool, ContactGlobalStat::Unread, event.timestamp, false).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_contact_resolved<E: Executor>(
    event: Event<Resolved>,
    action: Action<'_, GlobalStatView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(_data) => {}
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let Some(contact) = super::admin::find(&pool, &event.aggregator_id).await? else {
                return Ok(());
            };

            if !contact.is_unread() {
                return Ok(());
            }

            update(&pool, ContactGlobalStat::Unread, GLOBAL_TIMESTAMP, false).await?;
            update(&pool, ContactGlobalStat::Unread, event.timestamp, false).await?;
        }
    };

    Ok(())
}
fn to_day_string(timestamp: u64) -> anyhow::Result<String> {
    let date = UtcDateTime::from_unix_timestamp(timestamp.try_into()?)?;

    Ok(format!("{}-{}-{}", date.year(), date.month(), date.day()))
}

async fn update(
    pool: &SqlitePool,
    col: ContactGlobalStat,
    timestamp: u64,
    add: bool,
) -> anyhow::Result<()> {
    let day = to_day_string(timestamp)?;
    let (default_value, expr) = if add {
        (1, Expr::col(col.clone()).add(1))
    } else {
        (0, Expr::col(col.clone()).sub(1))
    };
    let statement = Query::insert()
        .into_table(ContactGlobalStat::Table)
        .columns([
            ContactGlobalStat::Day,
            ContactGlobalStat::CreatedAt,
            col.clone(),
        ])
        .values([day.into(), default_value.into(), timestamp.into()])?
        .on_conflict(
            OnConflict::column(ContactGlobalStat::Day)
                .value(col, expr)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}

async fn update_submitted(pool: &SqlitePool, timestamp: u64) -> anyhow::Result<()> {
    let day = to_day_string(timestamp)?;
    let statement = Query::insert()
        .into_table(ContactGlobalStat::Table)
        .columns([
            ContactGlobalStat::Day,
            ContactGlobalStat::Total,
            ContactGlobalStat::Today,
            ContactGlobalStat::Unread,
            ContactGlobalStat::CreatedAt,
        ])
        .values([day.into(), 1.into(), 1.into(), 1.into(), timestamp.into()])?
        .on_conflict(
            OnConflict::column(ContactGlobalStat::Day)
                .value(
                    ContactGlobalStat::Total,
                    Expr::col(ContactGlobalStat::Total).add(1),
                )
                .value(
                    ContactGlobalStat::Today,
                    Expr::col(ContactGlobalStat::Today).add(1),
                )
                .value(
                    ContactGlobalStat::Unread,
                    Expr::col(ContactGlobalStat::Unread).add(1),
                )
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(pool).await?;
    Ok(())
}
