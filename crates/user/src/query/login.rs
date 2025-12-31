use evento::{Action, Executor, Projection, SubscriptionBuilder, metadata::Event};
use imkitchen_db::table::UserLogin;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};

use crate::{
    Activated, LoggedIn, Logout, MadeAdmin, Role, State, Suspended, User, UsernameChanged,
    password::ResetCompleted, subscription::LifePremiumToggled,
};

#[derive(Default, Clone, Debug, FromRow)]
pub struct Login {
    pub id: String,
    pub user_agent: String,
    pub role: sqlx::types::Text<Role>,
    pub state: sqlx::types::Text<State>,
    pub username: Option<String>,
    pub subscription_expire_at: u64,
}

#[derive(Default, Clone, Debug)]
pub struct LoginView {
    pub id: String,
    pub logins: Vec<Login>,
}

impl Login {
    pub fn is_admin(&self) -> bool {
        self.role.0 == Role::Admin
    }

    pub fn is_premium(&self) -> bool {
        let Ok(now): Result<u64, _> = time::UtcDateTime::now().unix_timestamp().try_into() else {
            return false;
        };

        self.subscription_expire_at > now
    }

    pub fn username(&self) -> String {
        self.username.to_owned().unwrap_or("john_doe".to_owned())
    }
}

pub fn create_projection<E: Executor>() -> Projection<LoginView, E> {
    Projection::new("user-login-view")
        .handler(handle_logged_in())
        .handler(handle_logout())
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
        .handler(handle_reset_completed())
        .handler(handle_username_changed())
        .handler(handle_life_premium_toggled())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<LoginView>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .no_safety_check()
        .load::<User>(&id)
        .data(pool.clone())
        .execute(executor)
        .await?
        .map(|r| r.item))
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<LoginView, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::snapshot]
async fn restore(
    context: &evento::context::RwContext,
    id: String,
    _aggregators: &std::collections::HashMap<String, String>,
) -> anyhow::Result<Option<LoginView>> {
    let pool = context.extract::<SqlitePool>();
    let statement = Query::select()
        .columns([
            UserLogin::Id,
            UserLogin::Role,
            UserLogin::State,
            UserLogin::UserAgent,
            UserLogin::Username,
            UserLogin::SubscriptionExpireAt,
        ])
        .from(UserLogin::Table)
        .and_where(Expr::col(UserLogin::UserId).eq(&id))
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let logins: Vec<Login> = sqlx::query_as_with(&sql, values).fetch_all(&pool).await?;

    Ok(Some(LoginView { id, logins }))
}

#[evento::handler]
async fn handle_username_changed<E: Executor>(
    event: Event<UsernameChanged>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            for login in data.logins.iter_mut() {
                login.username = Some(event.data.value.to_owned());
            }
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            update(
                &pool,
                &event.aggregator_id,
                UserLogin::Username,
                event.data.value.to_owned(),
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_logged_in<E: Executor>(
    event: Event<LoggedIn>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.id = event.aggregator_id.to_owned();
            data.logins
                .retain(|r| r.user_agent != event.data.user_agent);
            data.logins.push(Login {
                id: event.data.access_id,
                role: sqlx::types::Text(event.data.role),
                state: sqlx::types::Text(event.data.state),
                subscription_expire_at: event.data.subscription_expire_at,
                username: event.data.username,
                user_agent: event.data.user_agent,
            });
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            let statement = Query::insert()
                .columns([
                    UserLogin::Id,
                    UserLogin::UserId,
                    UserLogin::Role,
                    UserLogin::State,
                    UserLogin::SubscriptionExpireAt,
                    UserLogin::Username,
                    UserLogin::UserAgent,
                ])
                .into_table(UserLogin::Table)
                .values([
                    event.data.access_id.to_owned().into(),
                    event.aggregator_id.to_string().into(),
                    event.data.role.to_string().into(),
                    event.data.state.to_string().into(),
                    event.data.subscription_expire_at.into(),
                    event.data.username.into(),
                    event.data.user_agent.into(),
                ])?
                .on_conflict(
                    OnConflict::columns([UserLogin::UserId, UserLogin::UserAgent])
                        .update_column(UserLogin::Id)
                        .to_owned(),
                )
                .to_owned();
            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_logout<E: Executor>(
    event: Event<Logout>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.logins.retain(|r| r.id != event.data.access_id);
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            delete(
                &pool,
                &event.aggregator_id,
                Some(event.data.access_id.to_owned()),
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_reset_completed<E: Executor>(
    event: Event<ResetCompleted>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            if data.id == event.metadata.user()? {
                data.logins = vec![];
            }
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            delete(&pool, &event.aggregator_id, None).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_made_admin<E: Executor>(
    event: Event<MadeAdmin>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            for login in data.logins.iter_mut() {
                login.role.0 = Role::Admin;
            }
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            update(
                &pool,
                &event.aggregator_id,
                UserLogin::Role,
                Role::Admin.to_string(),
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_actived<E: Executor>(
    event: Event<Activated>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            for login in data.logins.iter_mut() {
                login.state.0 = State::Active;
            }
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            update(
                &pool,
                &event.aggregator_id,
                UserLogin::State,
                State::Active.to_string(),
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_susended<E: Executor>(
    event: Event<Suspended>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            for login in data.logins.iter_mut() {
                login.state.0 = State::Suspended;
            }
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            update(
                &pool,
                &event.aggregator_id,
                UserLogin::State,
                State::Suspended.to_string(),
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_life_premium_toggled<E: Executor>(
    event: Event<LifePremiumToggled>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            for login in data.logins.iter_mut() {
                login.subscription_expire_at = event.data.expire_at;
            }
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            update(
                &pool,
                &event.aggregator_id,
                UserLogin::SubscriptionExpireAt,
                event.data.expire_at,
            )
            .await?;
        }
    };

    Ok(())
}

async fn update(
    pool: &SqlitePool,
    id: impl Into<String>,
    col: UserLogin,
    value: impl Into<Expr>,
) -> anyhow::Result<()> {
    let statement = Query::update()
        .table(UserLogin::Table)
        .and_where(Expr::col(UserLogin::UserId).eq(id.into()))
        .value(col, value)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}

async fn delete(
    pool: &SqlitePool,
    user_id: impl Into<String>,
    id: impl Into<Option<String>>,
) -> anyhow::Result<()> {
    let mut statement = Query::delete()
        .from_table(UserLogin::Table)
        .and_where(Expr::col(UserLogin::UserId).eq(user_id.into()))
        .to_owned();

    if let Some(id) = id.into() {
        statement.and_where(Expr::col(UserLogin::Id).eq(id));
    }

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}
