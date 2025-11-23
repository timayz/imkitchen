use crate::{Contact, FormSubmitted, MarkedReadAndReply, Reopened, Resolved};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::ContactList;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

pub fn subscribe_list<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("contact-list")
        .handler(handle_form_submmited())
        .handler(handle_reopened())
        .handler(handle_marked_read_and_reply())
        .handler(handle_resolved())
        .handler_check_off()
}

#[evento::handler(Contact)]
async fn handle_form_submmited<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<FormSubmitted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::insert()
        .into_table(ContactList::Table)
        .columns([
            ContactList::Id,
            ContactList::Email,
            ContactList::Status,
            ContactList::Subject,
            ContactList::Message,
            ContactList::Name,
            ContactList::CreatedAt,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.data.email.to_owned().into(),
            event.data.status.to_string().into(),
            event.data.subject.to_string().into(),
            event.data.message.to_owned().into(),
            event.data.name.to_owned().into(),
            event.timestamp.into(),
        ])
        .to_owned();
    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Contact)]
async fn handle_marked_read_and_reply<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MarkedReadAndReply>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(ContactList::Table)
        .values([(ContactList::Status, event.data.status.to_string().into())])
        .and_where(Expr::col(ContactList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Contact)]
async fn handle_resolved<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Resolved>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(ContactList::Table)
        .values([(ContactList::Status, event.data.status.to_string().into())])
        .and_where(Expr::col(ContactList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Contact)]
async fn handle_reopened<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Reopened>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(ContactList::Table)
        .values([(ContactList::Status, event.data.status.to_string().into())])
        .and_where(Expr::col(ContactList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
