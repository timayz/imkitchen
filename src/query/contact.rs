use std::{fmt::Display, str::FromStr};

use bincode::{Decode, Encode};
use evento::{
    AggregatorName, Executor, SubscribeBuilder,
    cursor::{Args, ReadResult},
    sql::Reader,
};
use imkitchen_contact::{
    Contact as ContactAggregator, ContactStatus, ContactSubject, FormSubmitted, MarkedReadAndReply,
    Reopened, Resolved,
};
use imkitchen_db::table::ContactPjt;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::prelude::FromRow;

#[derive(Debug, Encode, Decode)]
pub struct ContactCursor {
    pub i: String,
    pub v: i64,
}

#[derive(Debug, Default, Deserialize, FromRow)]
pub struct Contact {
    pub id: String,
    pub email: String,
    pub name: String,
    pub subject: String,
    pub message: String,
    pub status: String,
    pub created_at: i64,
}

impl Contact {
    pub fn is_unread(&self) -> bool {
        self.status == ContactStatus::Unread.to_string()
    }

    pub fn is_read(&self) -> bool {
        self.status == ContactStatus::Read.to_string()
    }

    pub fn is_resolved(&self) -> bool {
        self.status == ContactStatus::Resolved.to_string()
    }

    pub fn created_at(&self) -> String {
        super::format_relative_time(self.created_at as u64)
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

impl evento::cursor::Cursor for Contact {
    type T = ContactCursor;

    fn serialize(&self) -> Self::T {
        Self::T {
            i: self.id.to_owned(),
            v: self.created_at,
        }
    }
}

impl evento::sql::Bind for Contact {
    type T = ContactPjt;
    type I = [Self::T; 2];
    type V = [Expr; 2];
    type Cursor = Self;

    fn columns() -> Self::I {
        [ContactPjt::CreatedAt, ContactPjt::Id]
    }

    fn values(
        cursor: <<Self as evento::sql::Bind>::Cursor as evento::cursor::Cursor>::T,
    ) -> Self::V {
        [cursor.v.into(), cursor.i.into()]
    }
}

#[derive(Debug, Deserialize)]
pub enum ContactSortBy {
    MostRecent,
    OldestFirst,
}

impl Display for ContactSortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for ContactSortBy {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MostRecent" => Ok(Self::MostRecent),
            "OldestFirst" => Ok(Self::OldestFirst),
            _ => Err(()),
        }
    }
}

pub struct ContactInput {
    pub status: Option<ContactStatus>,
    pub subject: Option<ContactSubject>,
    pub sort_by: ContactSortBy,
    pub args: Args,
}

pub async fn query_contacts(
    pool: &sqlx::SqlitePool,
    input: ContactInput,
) -> anyhow::Result<ReadResult<Contact>> {
    let mut statment = Query::select()
        .columns([
            ContactPjt::Id,
            ContactPjt::Email,
            ContactPjt::Status,
            ContactPjt::Subject,
            ContactPjt::Message,
            ContactPjt::Name,
            ContactPjt::CreatedAt,
        ])
        .from(ContactPjt::Table)
        .to_owned();

    if let Some(subject) = input.subject {
        statment.and_where(Expr::col(ContactPjt::Subject).eq(subject.to_string()));
    }

    if let Some(status) = input.status {
        statment.and_where(Expr::col(ContactPjt::Status).eq(status.to_string()));
    }

    let mut reader = Reader::new(statment);

    if matches!(input.sort_by, ContactSortBy::MostRecent) {
        reader.desc();
    }

    Ok(reader
        .args(input.args)
        .execute::<_, Contact, _>(pool)
        .await?)
}

pub async fn query_contact_by_id(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
) -> anyhow::Result<Contact> {
    let statment = Query::select()
        .columns([
            ContactPjt::Id,
            ContactPjt::Email,
            ContactPjt::Status,
            ContactPjt::Subject,
            ContactPjt::Message,
            ContactPjt::Name,
            ContactPjt::CreatedAt,
        ])
        .from(ContactPjt::Table)
        .and_where(Expr::col(ContactPjt::Id).eq(id.into()))
        .limit(1)
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with::<_, Contact, _>(&sql, values)
        .fetch_one(pool)
        .await?)
}

pub fn subscribe_contact<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("contact-query")
        .handler(handle_form_submmited())
        .handler(handle_reopened())
        .handler(handle_marked_read_and_reply())
        .handler(handle_resolved())
}

#[evento::handler(ContactAggregator)]
async fn handle_form_submmited<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<FormSubmitted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::insert()
        .into_table(ContactPjt::Table)
        .columns([
            ContactPjt::Id,
            ContactPjt::Email,
            ContactPjt::Status,
            ContactPjt::Subject,
            ContactPjt::Message,
            ContactPjt::Name,
            ContactPjt::CreatedAt,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.data.email.to_owned().into(),
            event.data.status.to_owned().into(),
            event.data.subject.to_owned().into(),
            event.data.message.to_owned().into(),
            event.data.name.to_owned().into(),
            event.timestamp.into(),
        ])
        .to_owned();
    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(ContactAggregator)]
async fn handle_marked_read_and_reply<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MarkedReadAndReply>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(ContactPjt::Table)
        .values([(ContactPjt::Status, event.data.status.to_owned().into())])
        .and_where(Expr::col(ContactPjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(ContactAggregator)]
async fn handle_resolved<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Resolved>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(ContactPjt::Table)
        .values([(ContactPjt::Status, event.data.status.to_owned().into())])
        .and_where(Expr::col(ContactPjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(ContactAggregator)]
async fn handle_reopened<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Reopened>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(ContactPjt::Table)
        .values([(ContactPjt::Status, event.data.status.to_owned().into())])
        .and_where(Expr::col(ContactPjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
