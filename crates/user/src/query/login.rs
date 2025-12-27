use evento::{
    Action, Aggregator, Executor, LoadResult, Projection, SubscriptionBuilder, metadata::Event,
};
use sqlx::SqlitePool;

use crate::{Activated, LoggedIn, Logout, MadeAdmin, Registered, Role, State, Suspended, User};

#[derive(Default, Clone, Debug)]
pub struct LoginRow {
    pub id: String,
    pub user_agent: String,
}

#[derive(Default, Clone, Debug)]
pub struct LoginView {
    pub id: String,
    pub rows: Vec<LoginRow>,
    pub role: Role,
    pub state: State,
    pub username: Option<String>,
    pub subscription_expire_at: u64,
}

impl LoginView {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }

    pub fn is_premium(&self) -> bool {
        false
    }

    pub fn username(&self) -> String {
        self.username.to_owned().unwrap_or("john_doe".to_owned())
    }
}

fn create_projection<E: Executor>() -> Projection<LoginView, E> {
    Projection::new("user-login-view")
        .handler(handle_logged_in())
        .handler(handle_logout())
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<LoginView>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .load::<User>(&id)
        .data(pool.clone())
        .execute(executor)
        .await?
        .map(|r| r.item))
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<LoginView, E> {
    create_projection().subscription()
}

#[evento::snapshot]
async fn restore(
    _context: &evento::context::RwContext,
    _id: String,
) -> anyhow::Result<Option<LoginView>> {
    Ok(None)
}

#[evento::handler]
async fn handle_registered<E: Executor>(
    event: Event<Registered>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.id = event.aggregator_id.to_owned();
            data.role = Role::User;
            data.state = State::Active;
        }
        Action::Handle(_context) => {}
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
            data.rows.retain(|r| r.user_agent != event.data.user_agent);
            data.rows.push(LoginRow {
                id: event.data.access_id,
                user_agent: event.data.user_agent,
            });
        }
        Action::Handle(_context) => {}
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
            data.rows.retain(|r| r.id != event.data.access_id);
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}

#[evento::handler]
async fn handle_made_admin<E: Executor>(
    _event: Event<MadeAdmin>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.role = Role::Admin;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}

#[evento::handler]
async fn handle_actived<E: Executor>(
    _event: Event<Activated>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.state = State::Active;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}

#[evento::handler]
async fn handle_susended<E: Executor>(
    _event: Event<Suspended>,
    action: Action<'_, LoginView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.state = State::Suspended;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}
