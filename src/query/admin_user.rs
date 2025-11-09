use std::{fmt::Display, ops::Deref};

use bincode::{Decode, Encode};
use evento::{
    AggregatorName, Executor, SubscribeBuilder,
    cursor::{Args, Edge, ReadResult},
    sql::Reader,
};
use imkitchen_db::table::AdminUserPjt;
use imkitchen_shared::Event;
use imkitchen_user::{
    Activated, LoggedIn, MadeAdmin, RegistrationFailed, RegistrationRequested,
    RegistrationSucceeded, Suspended, User,
    subscription::{LifePremiumToggled, UserSubscription},
};
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::prelude::FromRow;

// #[derive(Debug, Encode, Decode)]
// pub struct AdminUserStringCursor {
//     pub i: String,
//     pub v: String,
// }

#[derive(Debug, Encode, Decode)]
pub struct AdminUserIntCursor {
    pub i: String,
    pub v: i64,
}

#[derive(Debug, Default, Deserialize, FromRow)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub username: Option<String>,
    pub account_type: String,
    pub status: String,
    pub total_recipes_count: i64,
    pub shared_recipes_count: i64,
    pub created_at: i64,
}

impl AdminUser {
    pub fn is_admin(&self) -> bool {
        self.account_type == AdminUserAccountType::Admin.to_string()
    }

    pub fn is_free_tier(&self) -> bool {
        self.account_type == AdminUserAccountType::FreeTier.to_string()
    }

    pub fn is_premium(&self) -> bool {
        self.account_type == AdminUserAccountType::Premium.to_string()
    }

    pub fn is_active(&self) -> bool {
        self.status == AdminUserStatus::Active.to_string()
    }

    pub fn is_suspended(&self) -> bool {
        self.status == AdminUserStatus::Suspended.to_string()
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
}

pub struct AdminUserSortByRecentlyJoined(AdminUser);

impl Deref for AdminUserSortByRecentlyJoined {
    type Target = AdminUser;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl evento::cursor::Cursor for AdminUserSortByRecentlyJoined {
    type T = AdminUserIntCursor;

    fn serialize(&self) -> Self::T {
        Self::T {
            i: self.id.to_owned(),
            v: self.created_at,
        }
    }
}

impl evento::sql::Bind for AdminUserSortByRecentlyJoined {
    type T = AdminUserPjt;
    type I = [Self::T; 2];
    type V = [Expr; 2];
    type Cursor = Self;

    fn columns() -> Self::I {
        [AdminUserPjt::CreatedAt, AdminUserPjt::Id]
    }

    fn values(
        cursor: <<Self as evento::sql::Bind>::Cursor as evento::cursor::Cursor>::T,
    ) -> Self::V {
        [cursor.v.into(), cursor.i.into()]
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for AdminUserSortByRecentlyJoined {
    fn from_row(row: &'_ sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let row = AdminUser::from_row(row)?;

        Ok(Self(row))
    }
}

#[derive(Debug, Deserialize)]
pub enum AdminUserStatus {
    Active,
    Suspended,
    Inactive,
}

impl Display for AdminUserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize)]
pub enum AdminUserAccountType {
    FreeTier,
    Premium,
    Admin,
}

impl Display for AdminUserAccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize)]
pub enum AdminUserSortBy {
    RecentlyJoined,
    Name,
    MostRecipes,
    MostActive,
}

pub struct AdminUserInput {
    pub status: Option<AdminUserStatus>,
    pub account_type: Option<AdminUserAccountType>,
    pub sort_by: AdminUserSortBy,
    pub args: Args,
}

pub async fn query_admin_users(
    pool: &sqlx::SqlitePool,
    input: AdminUserInput,
) -> anyhow::Result<ReadResult<AdminUser>> {
    let mut statment = Query::select()
        .columns([
            AdminUserPjt::Id,
            AdminUserPjt::Email,
            AdminUserPjt::FullName,
            AdminUserPjt::Username,
            AdminUserPjt::Status,
            AdminUserPjt::AccountType,
            AdminUserPjt::TotalRecipesCount,
            AdminUserPjt::SharedRecipesCount,
            AdminUserPjt::TotalActiveCount,
            AdminUserPjt::CreatedAt,
        ])
        .from(AdminUserPjt::Table)
        .to_owned();

    if let Some(account_type) = input.account_type {
        statment.and_where(Expr::col(AdminUserPjt::AccountType).eq(account_type.to_string()));
    }

    if let Some(status) = input.status {
        statment.and_where(Expr::col(AdminUserPjt::Status).eq(status.to_string()));
    }

    match input.sort_by {
        AdminUserSortBy::RecentlyJoined => {
            let result = Reader::new(statment)
                .desc()
                .args(input.args)
                .execute::<_, AdminUserSortByRecentlyJoined, _>(pool)
                .await?;

            Ok(ReadResult {
                page_info: result.page_info,
                edges: result
                    .edges
                    .into_iter()
                    .map(|e| Edge {
                        cursor: e.cursor.to_owned(),
                        node: e.node.0,
                    })
                    .collect(),
            })
        }
        _ => todo!(),
    }
}

pub async fn query_admin_user_by_id(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
) -> anyhow::Result<AdminUser> {
    let statment = Query::select()
        .columns([
            AdminUserPjt::Id,
            AdminUserPjt::Email,
            AdminUserPjt::FullName,
            AdminUserPjt::Username,
            AdminUserPjt::Status,
            AdminUserPjt::AccountType,
            AdminUserPjt::TotalRecipesCount,
            AdminUserPjt::SharedRecipesCount,
            AdminUserPjt::TotalActiveCount,
            AdminUserPjt::CreatedAt,
        ])
        .from(AdminUserPjt::Table)
        .and_where(Expr::col(AdminUserPjt::Id).eq(id.into()))
        .limit(1)
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with::<_, AdminUser, _>(&sql, values)
        .fetch_one(pool)
        .await?)
}

pub fn subscribe_admin_user<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("admin-user-query")
        .handler(handle_registration_succeeded())
        .handler(handle_suspended())
        .handler(handle_activated())
        .handler(handle_made_admin())
        .handler(handle_toggle_life_premium())
        .skip::<User, RegistrationRequested>()
        .skip::<User, LoggedIn>()
        .skip::<User, RegistrationFailed>()
}

#[evento::handler(User)]
async fn handle_registration_succeeded<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<RegistrationSucceeded>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::insert()
        .into_table(AdminUserPjt::Table)
        .columns([
            AdminUserPjt::Id,
            AdminUserPjt::Email,
            AdminUserPjt::Status,
            AdminUserPjt::AccountType,
            AdminUserPjt::CreatedAt,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.data.email.to_owned().into(),
            AdminUserStatus::Active.to_string().into(),
            AdminUserAccountType::FreeTier.to_string().into(),
            event.timestamp.into(),
        ])
        .to_owned();
    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(User)]
async fn handle_suspended<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Suspended>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(AdminUserPjt::Table)
        .values([(
            AdminUserPjt::Status,
            AdminUserStatus::Suspended.to_string().into(),
        )])
        .and_where(Expr::col(AdminUserPjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(User)]
async fn handle_activated<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Activated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(AdminUserPjt::Table)
        .values([(
            AdminUserPjt::Status,
            AdminUserStatus::Active.to_string().into(),
        )])
        .and_where(Expr::col(AdminUserPjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(User)]
async fn handle_made_admin<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MadeAdmin>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(AdminUserPjt::Table)
        .values([(
            AdminUserPjt::AccountType,
            AdminUserAccountType::Admin.to_string().into(),
        )])
        .and_where(Expr::col(AdminUserPjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(UserSubscription)]
async fn handle_toggle_life_premium<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<LifePremiumToggled>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let account_type = if event.data.expire_at > 0 {
        AdminUserAccountType::Premium
    } else {
        AdminUserAccountType::FreeTier
    };
    let statment = Query::update()
        .table(AdminUserPjt::Table)
        .values([(AdminUserPjt::AccountType, account_type.to_string().into())])
        .and_where(Expr::col(AdminUserPjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
