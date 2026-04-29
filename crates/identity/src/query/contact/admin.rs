use bitcode::{Decode, Encode};
use evento::{
    Cursor, Executor, Projection, Snapshot,
    cursor::{Args, ReadResult},
    metadata::Event,
    sql::Reader,
};
use imkitchen_db::table::{ContactAdmin, ContactAdminFts};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::SqlitePool;
use sqlx::prelude::FromRow;
use strum::{AsRefStr, Display, EnumString, VariantArray};

use imkitchen_shared::contact::{
    Contact, FormSubmitted, MarkedReadAndReply, Reopened, Resolved, Status, Subject,
};

#[evento::projection(Debug, FromRow, Cursor)]
pub struct AdminView {
    #[cursor(ContactAdmin::Id, 1)]
    pub id: String,
    pub email: String,
    pub name: String,
    pub subject: sqlx::types::Text<Subject>,
    pub message: String,
    pub status: sqlx::types::Text<Status>,
    #[cursor(ContactAdmin::CreatedAt, 2)]
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

pub struct FilterQuery {
    pub status: Option<Status>,
    pub subject: Option<Subject>,
    pub search: Option<String>,
    pub sort_by: SortBy,
    pub args: Args,
}

impl<E: Executor> super::Query<E> {
    pub async fn filter_admin(&self, input: FilterQuery) -> anyhow::Result<ReadResult<AdminView>> {
        let mut statement = sea_query::Query::select()
            .columns([
                ContactAdmin::Id,
                ContactAdmin::Cursor,
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

        if let Some(search) = input.search {
            statement.and_where(
                Expr::col(ContactAdmin::Id).in_subquery(
                    Query::select()
                        .column(ContactAdminFts::Id)
                        .from(ContactAdminFts::Table)
                        .and_where(Expr::cust(format!("contact_admin_fts MATCH '{search}*'")))
                        .order_by(ContactAdminFts::Rank, sea_query::Order::Asc)
                        .limit(20)
                        .take(),
                ),
            );
        }

        let mut reader = Reader::new(statement);

        if matches!(input.sort_by, SortBy::MostRecent) {
            reader.desc();
        }

        reader
            .args(input.args)
            .execute::<_, AdminView, _>(&self.read_db)
            .await
    }

    pub async fn find_admin(&self, id: impl Into<String>) -> anyhow::Result<Option<AdminView>> {
        find(&self.read_db, id).await
    }

    pub async fn admin(&self, id: impl Into<String>) -> Result<Option<AdminView>, anyhow::Error> {
        load(&self.executor, &self.read_db, &self.write_db, id).await
    }
}

pub(crate) async fn load<E: Executor>(
    executor: &E,
    read_db: &SqlitePool,
    write_db: &SqlitePool,
    id: impl Into<String>,
) -> Result<Option<AdminView>, anyhow::Error> {
    create_projection(id)
        .data((read_db.clone(), write_db.clone()))
        .execute(executor)
        .await
}

pub(crate) async fn find(
    pool: &SqlitePool,
    id: impl Into<String>,
) -> anyhow::Result<Option<AdminView>> {
    let statement = sea_query::Query::select()
        .columns([
            ContactAdmin::Id,
            ContactAdmin::Cursor,
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

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, AdminView> {
    Projection::new::<Contact>(id)
        .handler(handle_form_submmited())
        .handler(handle_reopened())
        .handler(handle_marked_read_and_reply())
        .handler(handle_resolved())
}

impl<E: Executor> Snapshot<E> for AdminView {
    async fn restore(context: &evento::projection::Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let (read_db, _) = context.extract::<(SqlitePool, SqlitePool)>();
        find(&read_db, &context.id).await
    }

    async fn take_snapshot(
        &self,
        context: &evento::projection::Context<'_, E>,
    ) -> anyhow::Result<()> {
        let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
        let statement = sea_query::Query::insert()
            .into_table(ContactAdmin::Table)
            .columns([
                ContactAdmin::Id,
                ContactAdmin::Cursor,
                ContactAdmin::Email,
                ContactAdmin::Status,
                ContactAdmin::Subject,
                ContactAdmin::Message,
                ContactAdmin::Name,
                ContactAdmin::CreatedAt,
            ])
            .values([
                self.id.to_owned().into(),
                self.cursor.to_owned().into(),
                self.email.to_owned().into(),
                self.status.to_string().into(),
                self.subject.to_string().into(),
                self.message.to_owned().into(),
                self.name.to_owned().into(),
                self.created_at.into(),
            ])?
            .on_conflict(
                OnConflict::column(ContactAdmin::Id)
                    .update_columns([
                        ContactAdmin::Cursor,
                        ContactAdmin::Email,
                        ContactAdmin::Status,
                        ContactAdmin::Subject,
                        ContactAdmin::Message,
                        ContactAdmin::Name,
                        ContactAdmin::CreatedAt,
                    ])
                    .to_owned(),
            )
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(&write_db).await?;

        Ok(())
    }
}

#[evento::handler]
async fn handle_form_submmited(
    event: Event<FormSubmitted>,
    data: &mut AdminView,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.email = event.data.email.to_owned();
    data.status.0 = Status::Unread;
    data.subject.0 = event.data.subject.to_owned();
    data.message = event.data.message.to_owned();
    data.name = event.data.name.to_owned();
    data.created_at = event.timestamp;

    Ok(())
}

#[evento::handler]
async fn handle_marked_read_and_reply(
    _event: Event<MarkedReadAndReply>,
    data: &mut AdminView,
) -> anyhow::Result<()> {
    data.status.0 = Status::Read;

    Ok(())
}

#[evento::handler]
async fn handle_resolved(_event: Event<Resolved>, data: &mut AdminView) -> anyhow::Result<()> {
    data.status.0 = Status::Resolved;

    Ok(())
}

#[evento::handler]
async fn handle_reopened(_event: Event<Reopened>, data: &mut AdminView) -> anyhow::Result<()> {
    data.status.0 = Status::Read;

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
