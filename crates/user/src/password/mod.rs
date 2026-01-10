mod request;
mod reset;

use std::ops::Deref;

pub use request::*;
pub use reset::*;

use evento::{Executor, Projection, Snapshot, metadata::Event};
use imkitchen_shared::user::password::{self, ResetCompleted, ResetRequested};
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Command<E: Executor>(pub(crate) imkitchen_shared::State<E>);

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Command<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Password>> {
        create_projection(id).execute(&self.executor).await
    }
}

#[evento::projection]
pub struct Password {
    pub id: String,
    pub user_id: String,
    pub completed: bool,
    pub expire_at: u64,
}

fn create_projection(id: impl Into<String>) -> Projection<Password> {
    Projection::new::<password::Password>(id)
        .handler(handle_reset_requested())
        .handler(handle_reset_completed())
        .safety_check()
}

impl evento::ProjectionAggregator for Password {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

impl Snapshot for Password {}

#[evento::handler]
async fn handle_reset_requested(
    event: Event<ResetRequested>,
    data: &mut Password,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.user_id = event.data.user_id.to_owned();
    data.completed = false;
    data.expire_at = (OffsetDateTime::from_unix_timestamp(event.timestamp.try_into()?)?
        + time::Duration::minutes(15))
    .unix_timestamp()
    .try_into()?;

    Ok(())
}

#[evento::handler]
async fn handle_reset_completed(
    _event: Event<ResetCompleted>,
    data: &mut Password,
) -> anyhow::Result<()> {
    data.completed = true;

    Ok(())
}
