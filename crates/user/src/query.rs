use std::{
    ops::Deref,
    time::{SystemTime, UNIX_EPOCH},
};

use bincode::{Decode, Encode};
use evento::{
    cursor::{Args, ReadResult},
    sql::Reader,
};
use imkitchen_db::table::{UserList, UserStat};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

use crate::{Role, State, UserSortBy};

#[derive(Debug, Default, FromRow)]
pub struct UserListRow {
    pub id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub username: Option<String>,
    pub role: sqlx::types::Text<Role>,
    pub state: sqlx::types::Text<State>,
    pub total_recipes_count: i64,
    pub shared_recipes_count: i64,
    pub subscription_expire_at: u64,
    pub created_at: i64,
}

impl UserListRow {
    pub fn is_admin(&self) -> bool {
        self.role.0 == Role::Admin
    }

    pub fn is_premium(&self) -> bool {
        let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return false;
        };

        self.role.0 == Role::Admin || self.subscription_expire_at > now.as_secs()
    }

    pub fn is_active(&self) -> bool {
        self.state.0 == State::Active
    }

    pub fn is_suspended(&self) -> bool {
        self.state.0 == State::Suspended
    }

    pub fn joined_at(&self) -> String {
        let Ok(created_at) = time::UtcDateTime::from_unix_timestamp(self.created_at) else {
            return "".to_owned();
        };

        let Ok(format) = time::format_description::parse("[month repr:short] [day], [year]") else {
            return "".to_owned();
        };

        created_at.format(&format).unwrap_or_else(|_| "".to_owned())
    }

    pub fn short_name(&self) -> String {
        self.full_name
            .to_owned()
            .unwrap_or(self.email.to_string())
            .split(' ')
            .take(2)
            .map(|w| w.chars().next().unwrap_or('a').to_uppercase().to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(Clone)]
pub struct Query(pub sqlx::SqlitePool);

pub struct FilterQuery {
    pub state: Option<State>,
    pub role: Option<Role>,
    pub sort_by: UserSortBy,
    pub args: Args,
}

impl Query {
    pub async fn filter(&self, input: FilterQuery) -> anyhow::Result<ReadResult<UserListRow>> {
        let mut statment = sea_query::Query::select()
            .columns([
                UserList::Id,
                UserList::Email,
                UserList::FullName,
                UserList::Username,
                UserList::State,
                UserList::Role,
                UserList::SubscriptionExpireAt,
                UserList::TotalRecipesCount,
                UserList::SharedRecipesCount,
                UserList::TotalActiveCount,
                UserList::CreatedAt,
            ])
            .from(UserList::Table)
            .to_owned();

        if let Some(account_type) = input.role {
            statment.and_where(Expr::col(UserList::Role).eq(account_type.to_string()));
        }

        if let Some(status) = input.state {
            statment.and_where(Expr::col(UserList::State).eq(status.to_string()));
        }

        match input.sort_by {
            UserSortBy::RecentlyJoined => {
                let result = Reader::new(statment)
                    .desc()
                    .args(input.args)
                    .execute::<_, UserSortByRecentlyJoined, _>(&self.0)
                    .await?;

                Ok(result.map(|user| user.0))
            }
            _ => todo!(),
        }
    }

    pub async fn find(&self, id: impl Into<String>) -> anyhow::Result<Option<UserListRow>> {
        let statment = sea_query::Query::select()
            .columns([
                UserList::Id,
                UserList::Email,
                UserList::FullName,
                UserList::Username,
                UserList::State,
                UserList::Role,
                UserList::SubscriptionExpireAt,
                UserList::TotalRecipesCount,
                UserList::SharedRecipesCount,
                UserList::TotalActiveCount,
                UserList::CreatedAt,
            ])
            .from(UserList::Table)
            .and_where(Expr::col(UserList::Id).eq(id.into()))
            .limit(1)
            .to_owned();

        let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, UserListRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}

pub struct UserSortByRecentlyJoined(UserListRow);

impl Deref for UserSortByRecentlyJoined {
    type Target = UserListRow;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl evento::cursor::Cursor for UserSortByRecentlyJoined {
    type T = UserCursorInt;

    fn serialize(&self) -> Self::T {
        Self::T {
            i: self.id.to_owned(),
            v: self.created_at,
        }
    }
}

impl evento::sql::Bind for UserSortByRecentlyJoined {
    type T = UserList;
    type I = [Self::T; 2];
    type V = [Expr; 2];
    type Cursor = Self;

    fn columns() -> Self::I {
        [UserList::CreatedAt, UserList::Id]
    }

    fn values(
        cursor: <<Self as evento::sql::Bind>::Cursor as evento::cursor::Cursor>::T,
    ) -> Self::V {
        [cursor.v.into(), cursor.i.into()]
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for UserSortByRecentlyJoined {
    fn from_row(row: &'_ sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let row = UserListRow::from_row(row)?;

        Ok(Self(row))
    }
}

#[derive(Debug, Encode, Decode)]
pub struct UserCursorString {
    pub i: String,
    pub v: String,
}

#[derive(Debug, Encode, Decode)]
pub struct UserCursorInt {
    pub i: String,
    pub v: i64,
}

#[derive(Default, FromRow)]
pub struct UserStatRow {
    pub total: u32,
    pub premium: u32,
    pub suspended: u32,
}

impl Query {
    pub async fn find_stat(&self, day: u64) -> anyhow::Result<Option<UserStatRow>> {
        let statment = sea_query::Query::select()
            .columns([UserStat::Total, UserStat::Premium, UserStat::Suspended])
            .from(UserStat::Table)
            .and_where(Expr::col(UserStat::Day).eq(day))
            .to_owned();

        let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
        Ok(sqlx::query_as_with::<_, UserStatRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}
