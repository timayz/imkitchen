use crate::{Contact, FormSubmitted, MarkedReadAndReply, Reopened, Resolved};
use bincode::{Decode, Encode};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use evento::{
    cursor::{Args, ReadResult},
    sql::Reader,
};
use imkitchen_db::table::ContactList;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

use crate::{SortBy, Status, Subject};

#[derive(Debug, Encode, Decode)]
pub struct ContactQueryCursor {
    pub i: String,
    pub v: u64,
}

#[derive(Debug, Default, FromRow)]
pub struct ContactRow {
    pub id: String,
    pub email: String,
    pub name: String,
    pub subject: sqlx::types::Text<Subject>,
    pub message: String,
    pub status: sqlx::types::Text<Status>,
    pub created_at: u64,
}

impl ContactRow {
    pub fn is_unread(&self) -> bool {
        self.status.0 == Status::Unread
    }

    pub fn is_read(&self) -> bool {
        self.status.0 == Status::Read
    }

    pub fn is_resolved(&self) -> bool {
        self.status.0 == Status::Resolved
    }

    pub fn created_at(&self) -> String {
        imkitchen_shared::format_relative_time(self.created_at)
    }

    pub fn short_name(&self) -> String {
        self.name
            .split(' ')
            .take(2)
            .map(|w| w.chars().next().unwrap_or('a').to_uppercase().to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

impl evento::cursor::Cursor for ContactRow {
    type T = ContactQueryCursor;

    fn serialize(&self) -> Self::T {
        Self::T {
            i: self.id.to_owned(),
            v: self.created_at,
        }
    }
}

impl evento::sql::Bind for ContactRow {
    type T = ContactList;
    type I = [Self::T; 2];
    type V = [Expr; 2];
    type Cursor = Self;

    fn columns() -> Self::I {
        [ContactList::CreatedAt, ContactList::Id]
    }

    fn values(
        cursor: <<Self as evento::sql::Bind>::Cursor as evento::cursor::Cursor>::T,
    ) -> Self::V {
        [cursor.v.into(), cursor.i.into()]
    }
}

pub struct FilterQuery {
    pub status: Option<Status>,
    pub subject: Option<Subject>,
    pub sort_by: SortBy,
    pub args: Args,
}

impl super::Query {
    pub async fn filter(&self, input: FilterQuery) -> anyhow::Result<ReadResult<ContactRow>> {
        let mut statement = sea_query::Query::select()
            .columns([
                ContactList::Id,
                ContactList::Email,
                ContactList::Status,
                ContactList::Subject,
                ContactList::Message,
                ContactList::Name,
                ContactList::CreatedAt,
            ])
            .from(ContactList::Table)
            .to_owned();

        if let Some(subject) = input.subject {
            statement.and_where(Expr::col(ContactList::Subject).eq(subject.to_string()));
        }

        if let Some(status) = input.status {
            statement.and_where(Expr::col(ContactList::Status).eq(status.to_string()));
        }

        let mut reader = Reader::new(statement);

        if matches!(input.sort_by, SortBy::MostRecent) {
            reader.desc();
        }

        Ok(reader
            .args(input.args)
            .execute::<_, ContactRow, _>(&self.0)
            .await?)
    }

    pub async fn find(&self, id: impl Into<String>) -> anyhow::Result<Option<ContactRow>> {
        let statement = sea_query::Query::select()
            .columns([
                ContactList::Id,
                ContactList::Email,
                ContactList::Status,
                ContactList::Subject,
                ContactList::Message,
                ContactList::Name,
                ContactList::CreatedAt,
            ])
            .from(ContactList::Table)
            .and_where(Expr::col(ContactList::Id).eq(id.into()))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, ContactRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}

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
    let statement = Query::insert()
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
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Contact)]
async fn handle_marked_read_and_reply<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MarkedReadAndReply>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(ContactList::Table)
        .values([(ContactList::Status, event.data.status.to_string().into())])
        .and_where(Expr::col(ContactList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Contact)]
async fn handle_resolved<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Resolved>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(ContactList::Table)
        .values([(ContactList::Status, event.data.status.to_string().into())])
        .and_where(Expr::col(ContactList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Contact)]
async fn handle_reopened<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Reopened>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(ContactList::Table)
        .values([(ContactList::Status, event.data.status.to_string().into())])
        .and_where(Expr::col(ContactList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
