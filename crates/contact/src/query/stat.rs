use crate::{Contact, FormSubmitted, MarkedReadAndReply, Resolved};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::ContactStat;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

#[derive(Default, Debug, FromRow)]
pub struct Stat {
    pub total: u32,
    pub unread: u32,
    // pub today: u32,
    pub avg_response_time: u32,
    // pub avg_response_time_last_week: u8,
}

impl super::Query {
    pub async fn find_stat(&self, day: u64) -> anyhow::Result<Option<Stat>> {
        let statement = sea_query::Query::select()
            .columns([
                ContactStat::Total,
                ContactStat::Unread,
                ContactStat::AvgResponseTime,
            ])
            .from(ContactStat::Table)
            .and_where(Expr::col(ContactStat::Day).eq(day))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        Ok(sqlx::query_as_with::<_, Stat, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}

pub fn subscribe_stat<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("contact-stat")
        .handler(handle_contact_form_submitted())
        .handler(handle_contact_marked_read_and_reply())
        .handler(handle_contact_resolved())
        .handler_check_off()
}

#[evento::handler(Contact)]
async fn handle_contact_form_submitted<E: Executor>(
    context: &evento::Context<'_, E>,
    _event: Event<FormSubmitted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::insert()
        .into_table(ContactStat::Table)
        .columns([ContactStat::Day, ContactStat::Total, ContactStat::Unread])
        .values_panic([0.into(), 1.into(), 1.into()])
        .on_conflict(
            OnConflict::column(ContactStat::Day)
                .value(ContactStat::Total, Expr::col(ContactStat::Total).add(1))
                .value(ContactStat::Unread, Expr::col(ContactStat::Unread).add(1))
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Contact)]
async fn handle_contact_marked_read_and_reply<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MarkedReadAndReply>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let query = context.extract::<crate::Query>();
    let Some(contact) = query.find(&event.aggregator_id).await? else {
        return Ok(());
    };

    if !contact.is_unread() {
        return Ok(());
    }

    let statement = Query::insert()
        .into_table(ContactStat::Table)
        .columns([ContactStat::Day, ContactStat::Unread])
        .values_panic([0.into(), 1.into()])
        .on_conflict(
            OnConflict::column(ContactStat::Day)
                .value(ContactStat::Unread, Expr::col(ContactStat::Unread).sub(1))
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Contact)]
async fn handle_contact_resolved<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Resolved>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let query = context.extract::<crate::Query>();
    let Some(contact) = query.find(&event.aggregator_id).await? else {
        return Ok(());
    };

    if !contact.is_unread() {
        return Ok(());
    }

    let statement = Query::insert()
        .into_table(ContactStat::Table)
        .columns([ContactStat::Day, ContactStat::Unread])
        .values_panic([0.into(), 1.into()])
        .on_conflict(
            OnConflict::column(ContactStat::Day)
                .value(ContactStat::Unread, Expr::col(ContactStat::Unread).sub(1))
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
