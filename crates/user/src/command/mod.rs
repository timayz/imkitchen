use evento::{Executor, Projection, Snapshot, metadata::Event};
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
}

pub fn create_projection(id: impl Into<String>) -> Projection<CommandData> {
    Projection::new::<User>(id)
        .handler(handle_registered())
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
        .safety_check()
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<Command<'a, E>>, anyhow::Error> {
    let id = id.into();

    let Some(data) = create_projection(&id)
        .data(pool.clone())
        .execute(executor)
        .await?
    else {
        return Ok(None);
    };

    Ok(Some(Command::new(
        id,
        data.get_cursor_version()?,
        data,
        executor,
    )))
}

impl Snapshot for CommandData {
    // fn restore_version(&self) -> u16 {
    //     self.version
    // }
    //
    // fn restore_routing_key(&self) -> Option<String> {
    //     self.routing_key.to_owned()
    // }
    //
    // fn restore<'a>(
    //     context: &'a evento::context::RwContext,
    //     id: String,
    //     _aggregators: &'a std::collections::HashMap<String, String>,
    // ) -> std::pin::Pin<Box<dyn Future<Output = anyhow::Result<Option<Self>>> + Send + 'a>> {
    //     Box::pin(async {
    //         let pool = context.extract::<SqlitePool>();
    //         Ok(repository::find(&pool, FindType::Id(id))
    //             .await?
    //             .map(|row| CommandData {
    //                 role: row.role.0,
    //                 state: row.state.0,
    //                 version: row.version,
    //                 routing_key: row.routing_key,
    //             }))
    //     })
    // }
}

#[evento::handler]
async fn handle_registered(
    _event: Event<Registered>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.state = State::Active;
    data.role = Role::User;

    Ok(())
}

#[evento::handler]
async fn handle_made_admin(_event: Event<MadeAdmin>, data: &mut CommandData) -> anyhow::Result<()> {
    data.role = Role::Admin;

    Ok(())
}

#[evento::handler]
async fn handle_actived(_event: Event<Activated>, data: &mut CommandData) -> anyhow::Result<()> {
    data.state = State::Active;

    Ok(())
}

#[evento::handler]
async fn handle_susended(_event: Event<Suspended>, data: &mut CommandData) -> anyhow::Result<()> {
    data.state = State::Suspended;

    Ok(())
}
