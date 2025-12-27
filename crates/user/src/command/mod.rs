use evento::{
    Action, Aggregator, Executor, LoadResult, Projection, SubscriptionBuilder, metadata::Event,
};
use sqlx::SqlitePool;

use crate::{Activated, MadeAdmin, Registered, Role, State, Suspended, User};

mod activate;
mod login;
mod made_admin;
mod register;
mod repository;
mod set_username;
mod suspend;

pub use login::LoginInput;
pub use register::RegisterInput;

#[evento::command]
pub struct Command {
    pub role: Role,
    pub state: State,
}

fn create_projection<E: Executor>() -> Projection<CommandData, E> {
    Projection::new("user-command")
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<Command<'a, E>>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .load::<User>(&id)
        .data(pool.clone())
        .filter_events_by_name(false)
        .execute(executor)
        .await?
        .map(|loaded| Command::new(id, loaded, executor)))
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<CommandData, E> {
    create_projection().subscription()
}

#[evento::snapshot]
async fn restore(
    context: &evento::context::RwContext,
    id: String,
) -> anyhow::Result<Option<CommandData>> {
    let pool = context.extract::<SqlitePool>();
    Ok(repository::find(&pool, repository::FindType::Id(id))
        .await?
        .map(|row| CommandData {
            role: row.role.0,
            state: row.state.0,
        }))
}

#[evento::handler]
async fn handle_made_admin<E: Executor>(
    _event: Event<MadeAdmin>,
    action: Action<'_, CommandData, E>,
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
    action: Action<'_, CommandData, E>,
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
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.state = State::Suspended;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}
