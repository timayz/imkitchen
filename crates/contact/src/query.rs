use bincode::{Decode, Encode};
use evento::{
    cursor::{Args, ReadResult},
    sql::Reader,
};
use imkitchen_db::table::{ContactList, ContactStat};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
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

#[derive(Clone)]
pub struct Query(pub sqlx::SqlitePool);

pub struct FilterQuery {
    pub status: Option<Status>,
    pub subject: Option<Subject>,
    pub sort_by: SortBy,
    pub args: Args,
}

impl Query {
    pub async fn filter(&self, input: FilterQuery) -> anyhow::Result<ReadResult<ContactRow>> {
        let mut statment = sea_query::Query::select()
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
            statment.and_where(Expr::col(ContactList::Subject).eq(subject.to_string()));
        }

        if let Some(status) = input.status {
            statment.and_where(Expr::col(ContactList::Status).eq(status.to_string()));
        }

        let mut reader = Reader::new(statment);

        if matches!(input.sort_by, SortBy::MostRecent) {
            reader.desc();
        }

        Ok(reader
            .args(input.args)
            .execute::<_, ContactRow, _>(&self.0)
            .await?)
    }

    pub async fn find(&self, id: impl Into<String>) -> anyhow::Result<Option<ContactRow>> {
        let statment = sea_query::Query::select()
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

        let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, ContactRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}

#[derive(Default, Debug, FromRow)]
pub struct Stat {
    pub total: u32,
    pub unread: u32,
    // pub today: u32,
    pub avg_response_time: u32,
    // pub avg_response_time_last_week: u8,
}

impl Query {
    pub async fn find_stat(&self, day: u64) -> anyhow::Result<Option<Stat>> {
        let statment = sea_query::Query::select()
            .columns([
                ContactStat::Total,
                ContactStat::Unread,
                ContactStat::AvgResponseTime,
            ])
            .from(ContactStat::Table)
            .and_where(Expr::col(ContactStat::Day).eq(day))
            .to_owned();

        let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
        Ok(sqlx::query_as_with::<_, Stat, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}
