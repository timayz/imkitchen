use bitcode::{Decode, Encode};
use evento::{
    Action, Executor, Projection, SubscriptionBuilder,
    cursor::{Args, CursorUInt, ReadResult},
    metadata::Event,
    sql::Reader,
};
use imkitchen_db::table::ContactAdmin;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::SqlitePool;
use sqlx::prelude::FromRow;
use strum::{AsRefStr, Display, EnumString, VariantArray};

use crate::{Contact, FormSubmitted, MarkedReadAndReply, Reopened, Resolved, Status, Subject};

#[derive(Debug, Default, FromRow)]
pub struct AdminView {
    pub id: String,
    pub email: String,
    pub name: String,
    pub subject: sqlx::types::Text<Subject>,
    pub message: String,
    pub status: sqlx::types::Text<Status>,
    pub created_at: u64,
}

impl AdminView {
    pub fn is_unread(&self) -> bool {
        self.status.0 == Status::Unread
    }

    pub fn is_read(&self) -> bool {
        self.status.0 == Status::Read
    }

    pub fn is_resolved(&self) -> bool {
        self.status.0 == Status::Resolved
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

impl evento::cursor::Cursor for AdminView {
    type T = CursorUInt;

    fn serialize(&self) -> Self::T {
        Self::T {
            i: self.id.to_owned(),
            v: self.created_at,
        }
    }
}

impl evento::sql::Bind for AdminView {
    type T = ContactAdmin;
    type I = [Self::T; 2];
    type V = [Expr; 2];
    type Cursor = Self;

    fn columns() -> Self::I {
        [ContactAdmin::CreatedAt, ContactAdmin::Id]
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

pub async fn filter(
    pool: &SqlitePool,
    input: FilterQuery,
) -> anyhow::Result<ReadResult<AdminView>> {
    let mut statement = sea_query::Query::select()
        .columns([
            ContactAdmin::Id,
            ContactAdmin::Email,
            ContactAdmin::Status,
            ContactAdmin::Subject,
            ContactAdmin::Message,
            ContactAdmin::Name,
            ContactAdmin::CreatedAt,
        ])
        .from(ContactAdmin::Table)
        .to_owned();

    if let Some(subject) = input.subject {
        statement.and_where(Expr::col(ContactAdmin::Subject).eq(subject.to_string()));
    }

    if let Some(status) = input.status {
        statement.and_where(Expr::col(ContactAdmin::Status).eq(status.to_string()));
    }

    let mut reader = Reader::new(statement);

    if matches!(input.sort_by, SortBy::MostRecent) {
        reader.desc();
    }

    reader
        .args(input.args)
        .execute::<_, AdminView, _>(pool)
        .await
}

pub async fn find(pool: &SqlitePool, id: impl Into<String>) -> anyhow::Result<Option<AdminView>> {
    let statement = sea_query::Query::select()
        .columns([
            ContactAdmin::Id,
            ContactAdmin::Email,
            ContactAdmin::Status,
            ContactAdmin::Subject,
            ContactAdmin::Message,
            ContactAdmin::Name,
            ContactAdmin::CreatedAt,
        ])
        .from(ContactAdmin::Table)
        .and_where(Expr::col(ContactAdmin::Id).eq(id.into()))
        .limit(1)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with::<_, AdminView, _>(&sql, values)
        .fetch_optional(pool)
        .await?)
}

pub fn create_projection<E: Executor>() -> Projection<AdminView, E> {
    Projection::new("contact-admin-view")
        .handler(handle_form_submmited())
        .handler(handle_reopened())
        .handler(handle_marked_read_and_reply())
        .handler(handle_resolved())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<AdminView>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .no_safety_check()
        .load::<Contact>(id)
        .data(pool.clone())
        .execute(executor)
        .await?
        .map(|r| r.item))
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<AdminView, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::snapshot]
async fn restore(
    context: &evento::context::RwContext,
    id: String,
) -> anyhow::Result<Option<AdminView>> {
    let pool = context.extract::<SqlitePool>();
    find(&pool, id).await
}

#[evento::handler]
async fn handle_form_submmited<E: Executor>(
    event: Event<FormSubmitted>,
    action: Action<'_, AdminView, E>,
) -> anyhow::Result<()> {
    let status = Status::Unread;
    match action {
        Action::Apply(data) => {
            data.id = event.aggregator_id.to_owned();
            data.email = event.data.email.to_owned();
            data.status.0 = status;
            data.subject.0 = event.data.subject.to_owned();
            data.message = event.data.message.to_owned();
            data.name = event.data.name.to_owned();
            data.created_at = event.timestamp;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let statement = Query::insert()
                .into_table(ContactAdmin::Table)
                .columns([
                    ContactAdmin::Id,
                    ContactAdmin::Email,
                    ContactAdmin::Status,
                    ContactAdmin::Subject,
                    ContactAdmin::Message,
                    ContactAdmin::Name,
                    ContactAdmin::CreatedAt,
                ])
                .values_panic([
                    event.aggregator_id.to_owned().into(),
                    event.data.email.to_owned().into(),
                    status.to_string().into(),
                    event.data.subject.to_string().into(),
                    event.data.message.to_owned().into(),
                    event.data.name.to_owned().into(),
                    event.timestamp.into(),
                ])
                .to_owned();
            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_marked_read_and_reply<E: Executor>(
    event: Event<MarkedReadAndReply>,
    action: Action<'_, AdminView, E>,
) -> anyhow::Result<()> {
    let status = Status::Read;
    match action {
        Action::Apply(data) => {
            data.status.0 = status;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            update(
                &pool,
                UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    status: Some(status),
                },
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_resolved<E: Executor>(
    event: Event<Resolved>,
    action: Action<'_, AdminView, E>,
) -> anyhow::Result<()> {
    let status = Status::Resolved;
    match action {
        Action::Apply(data) => {
            data.status.0 = status;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            update(
                &pool,
                UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    status: Some(status),
                },
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_reopened<E: Executor>(
    event: Event<Reopened>,
    action: Action<'_, AdminView, E>,
) -> anyhow::Result<()> {
    let status = Status::Read;

    match action {
        Action::Apply(data) => {
            data.status.0 = status;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            update(
                &pool,
                UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    status: Some(status),
                },
            )
            .await?;
        }
    };

    Ok(())
}

#[derive(
    Encode,
    Decode,
    EnumString,
    Display,
    VariantArray,
    Default,
    Clone,
    Debug,
    PartialEq,
    Deserialize,
    AsRefStr,
)]
pub enum SortBy {
    #[default]
    MostRecent,
    OldestFirst,
}

struct UpdateInput {
    id: String,
    status: Option<Status>,
}

async fn update(pool: &SqlitePool, input: UpdateInput) -> anyhow::Result<()> {
    let mut statement = Query::update()
        .table(ContactAdmin::Table)
        .and_where(Expr::col(ContactAdmin::Id).eq(input.id))
        .to_owned();

    if let Some(status) = input.status {
        statement.value(ContactAdmin::Status, status.to_string());
    }

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}
