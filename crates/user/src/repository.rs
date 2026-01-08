use imkitchen_db::table::User;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};
use time::OffsetDateTime;

use imkitchen_shared::user::{Role, State};

#[derive(FromRow)]
pub struct UserRow {
    pub id: String,
    pub password: String,
    pub username: Option<String>,
    pub role: sqlx::types::Text<Role>,
    pub state: sqlx::types::Text<State>,
}

pub enum FindType {
    Id(String),
    Email(String),
}

pub(crate) async fn find(
    pool: &SqlitePool,
    arg_type: FindType,
) -> imkitchen_shared::Result<Option<UserRow>> {
    let mut statement = Query::select()
        .columns([
            User::Id,
            User::Password,
            User::Username,
            User::Role,
            User::State,
        ])
        .from(User::Table)
        .limit(1)
        .to_owned();

    match arg_type {
        FindType::Id(id) => statement.and_where(Expr::col(User::Id).eq(id)),
        FindType::Email(email) => statement.and_where(Expr::col(User::Email).eq(email)),
    };

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with::<_, UserRow, _>(&sql, values)
        .fetch_optional(pool)
        .await?)
}

pub(super) async fn create(
    pool: &SqlitePool,
    id: String,
    email: String,
    password: String,
) -> imkitchen_shared::Result<()> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let statement = Query::insert()
        .into_table(User::Table)
        .columns([
            User::Id,
            User::Email,
            User::Password,
            User::Role,
            User::State,
            User::CreatedAt,
        ])
        .values_panic([
            id.into(),
            email.into(),
            password.into(),
            Role::User.to_string().into(),
            State::Active.to_string().into(),
            now.into(),
        ])
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}

pub struct UpdateInput {
    pub id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
    pub state: Option<State>,
}

pub async fn update(pool: &SqlitePool, input: UpdateInput) -> imkitchen_shared::Result<()> {
    let mut statement = Query::update()
        .table(User::Table)
        .and_where(Expr::col(User::Id).eq(input.id))
        .to_owned();

    if let Some(username) = input.username {
        statement.value(User::Username, username);
    }

    if let Some(password) = input.password {
        statement.value(User::Password, password);
    }

    if let Some(role) = input.role {
        statement.value(User::Role, role.as_ref());
    }

    if let Some(state) = input.state {
        statement.value(User::State, state.as_ref());
    }

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}

pub async fn is_username_exists(
    pool: &SqlitePool,
    username: impl Into<String>,
) -> imkitchen_shared::Result<bool> {
    let statement = Query::select()
        .column(User::Id)
        .from(User::Table)
        .and_where(Expr::col(User::Username).eq(username.into()))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    let row = sqlx::query_as_with::<_, (String,), _>(&sql, values)
        .fetch_optional(pool)
        .await?;

    Ok(row.is_some())
}
