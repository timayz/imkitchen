use evento::{Executor, Projection, ProjectionCursor, Snapshot, cursor, metadata::Event};
use sqlx::{SqlitePool, prelude::FromRow};

use imkitchen_shared::user::{
    Activated, LoggedIn, Logout, MadeAdmin, Role, State, Suspended, User, UsernameChanged,
    password::ResetCompleted,
    subscription::{LifePremiumToggled, Subscription},
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
    pub cursor: cursor::Value,
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

pub fn create_projection(id: impl Into<String>) -> Projection<LoginView> {
    let id = id.into();

    Projection::new::<User>(&id)
        .aggregator::<Subscription>(id)
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

    create_projection(&id)
        .data(pool.clone())
        .execute(executor)
        .await
}

impl ProjectionCursor for LoginView {
    fn get_cursor(&self) -> cursor::Value {
        self.cursor.to_owned()
    }
    fn set_cursor(&mut self, v: &cursor::Value) {
        self.cursor = v.to_owned();
    }
}

impl Snapshot for LoginView {}

// #[evento::snapshot]
// async fn restore(
//     context: &evento::context::RwContext,
//     id: String,
//     _aggregators: &std::collections::HashMap<String, String>,
// ) -> anyhow::Result<Option<LoginView>> {
//     let pool = context.extract::<SqlitePool>();
//     let statement = Query::select()
//         .columns([
//             UserLogin::Id,
//             UserLogin::Role,
//             UserLogin::State,
//             UserLogin::UserAgent,
//             UserLogin::Username,
//             UserLogin::SubscriptionExpireAt,
//         ])
//         .from(UserLogin::Table)
//         .and_where(Expr::col(UserLogin::UserId).eq(&id))
//         .to_owned();
//     let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
//
//     let logins: Vec<Login> = sqlx::query_as_with(&sql, values).fetch_all(&pool).await?;
//
//     Ok(Some(LoginView { id, logins }))
// }

#[evento::handler]
async fn handle_username_changed(
    event: Event<UsernameChanged>,
    data: &mut LoginView,
) -> anyhow::Result<()> {
    for login in data.logins.iter_mut() {
        login.username = Some(event.data.value.to_owned());
    }

    Ok(())
}

#[evento::handler]
async fn handle_logged_in(event: Event<LoggedIn>, data: &mut LoginView) -> anyhow::Result<()> {
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

    Ok(())
}

#[evento::handler]
async fn handle_logout(event: Event<Logout>, data: &mut LoginView) -> anyhow::Result<()> {
    data.logins.retain(|r| r.id != event.data.access_id);

    Ok(())
}

#[evento::handler]
async fn handle_reset_completed(
    event: Event<ResetCompleted>,
    data: &mut LoginView,
) -> anyhow::Result<()> {
    if data.id == event.metadata.user()? {
        data.logins = vec![];
    }

    Ok(())
}

#[evento::handler]
async fn handle_made_admin(_event: Event<MadeAdmin>, data: &mut LoginView) -> anyhow::Result<()> {
    for login in data.logins.iter_mut() {
        login.role.0 = Role::Admin;
    }

    Ok(())
}

#[evento::handler]
async fn handle_actived(_event: Event<Activated>, data: &mut LoginView) -> anyhow::Result<()> {
    for login in data.logins.iter_mut() {
        login.state.0 = State::Active;
    }

    Ok(())
}

#[evento::handler]
async fn handle_susended(_event: Event<Suspended>, data: &mut LoginView) -> anyhow::Result<()> {
    for login in data.logins.iter_mut() {
        login.state.0 = State::Suspended;
    }

    Ok(())
}

#[evento::handler]
async fn handle_life_premium_toggled(
    event: Event<LifePremiumToggled>,
    data: &mut LoginView,
) -> anyhow::Result<()> {
    for login in data.logins.iter_mut() {
        login.subscription_expire_at = event.data.expire_at;
    }

    Ok(())
}

// async fn update(
//     pool: &SqlitePool,
//     id: impl Into<String>,
//     col: UserLogin,
//     value: impl Into<Expr>,
// ) -> anyhow::Result<()> {
//     let statement = Query::update()
//         .table(UserLogin::Table)
//         .and_where(Expr::col(UserLogin::UserId).eq(id.into()))
//         .value(col, value)
//         .to_owned();
//
//     let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
//     sqlx::query_with(&sql, values).execute(pool).await?;
//
//     Ok(())
// }
//
// async fn delete(
//     pool: &SqlitePool,
//     user_id: impl Into<String>,
//     id: impl Into<Option<String>>,
// ) -> anyhow::Result<()> {
//     let mut statement = Query::delete()
//         .from_table(UserLogin::Table)
//         .and_where(Expr::col(UserLogin::UserId).eq(user_id.into()))
//         .to_owned();
//
//     if let Some(id) = id.into() {
//         statement.and_where(Expr::col(UserLogin::Id).eq(id));
//     }
//
//     let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
//     sqlx::query_with(&sql, values).execute(pool).await?;
//
//     Ok(())
// }
