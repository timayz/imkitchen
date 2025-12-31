use evento::{Action, Executor, Projection, Snapshot, SubscriptionBuilder, metadata::Event};
use sqlx::SqlitePool;

use crate::{
    Activated, MadeAdmin, Registered, Role, State, Suspended, User,
    repository::{self, FindType},
};

mod activate;
mod login;
mod made_admin;
mod register;
mod set_username;
mod suspend;

pub use login::LoginInput;
pub use register::RegisterInput;
pub use set_username::SetUsernameInput;

#[evento::command]
pub struct Command {
    pub role: Role,
    pub state: State,
    pub version: u16,
    pub routing_key: Option<String>,
}

pub fn create_projection<E: Executor>() -> Projection<CommandData, E> {
    Projection::new("user-command")
        .handler(handle_registered())
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
        .no_safety_check()
        .load::<User>(&id)
        .data(pool.clone())
        .execute_all(executor)
        .await?
        .map(|loaded| Command::new(id, loaded, executor)))
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<CommandData, E> {
    create_projection().no_safety_check().subscription()
}

impl Snapshot for CommandData {
    fn restore_version(&self) -> u16 {
        self.version
    }

    fn restore_routing_key(&self) -> Option<String> {
        self.routing_key.to_owned()
    }

    fn restore<'a>(
        context: &'a evento::context::RwContext,
        id: String,
        _aggregators: &'a std::collections::HashMap<String, String>,
    ) -> std::pin::Pin<Box<dyn Future<Output = anyhow::Result<Option<Self>>> + Send + 'a>> {
        Box::pin(async {
            let pool = context.extract::<SqlitePool>();
            Ok(repository::find(&pool, FindType::Id(id))
                .await?
                .map(|row| CommandData {
                    role: row.role.0,
                    state: row.state.0,
                    version: row.version,
                    routing_key: row.routing_key,
                }))
        })
    }
}

// #[evento::snapshot]
// async fn restore(
//     context: &evento::context::RwContext,
//     id: String,
//     _aggregators: &std::collections::HashMap<String, String>,
// ) -> anyhow::Result<Option<CommandData>> {
//     let pool = context.extract::<SqlitePool>();
//     Ok(repository::find(&pool, FindType::Id(id))
//         .await?
//         .map(|row| CommandData {
//             role: row.role.0,
//             state: row.state.0,
//             version: row.version,
//             routing_key: row.routing_key,
//         }))
// }

#[evento::handler]
async fn handle_registered<E: Executor>(
    event: Event<Registered>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.state = State::Active;
            data.role = Role::User;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            repository::update(
                &pool,
                repository::UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    username: None,
                    password: None,
                    role: Some(Role::User),
                    state: Some(State::Active),
                    version: event.version,
                    routing_key: event.routing_key.to_owned(),
                },
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_made_admin<E: Executor>(
    event: Event<MadeAdmin>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.role = Role::Admin;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            repository::update(
                &pool,
                repository::UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    username: None,
                    password: None,
                    role: Some(Role::Admin),
                    state: None,
                    version: event.version,
                    routing_key: event.routing_key.to_owned(),
                },
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_actived<E: Executor>(
    event: Event<Activated>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.state = State::Active;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            repository::update(
                &pool,
                repository::UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    username: None,
                    password: None,
                    role: None,
                    state: Some(State::Active),
                    version: event.version,
                    routing_key: event.routing_key.to_owned(),
                },
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_susended<E: Executor>(
    event: Event<Suspended>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.state = State::Suspended;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            repository::update(
                &pool,
                repository::UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    username: None,
                    password: None,
                    role: None,
                    state: Some(State::Suspended),
                    version: event.version,
                    routing_key: event.routing_key.to_owned(),
                },
            )
            .await?;
        }
    };

    Ok(())
}
