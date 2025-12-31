use evento::{Action, Executor, Projection, Snapshot, SubscriptionBuilder, metadata::Event};
use sqlx::SqlitePool;

use crate::{Contact, FormSubmitted, MarkedReadAndReply, Reopened, Resolved, Status};

mod mark_read_and_replay;
mod reopen;
mod resolve;
mod submit_form;

pub use submit_form::SubmitFormInput;

#[evento::command]
pub struct Command {
    pub status: Status,
}

impl Snapshot for CommandData {}

pub fn create_projection<E: Executor>() -> Projection<CommandData, E> {
    Projection::new("contact-command")
        .handler(handle_form_submitted())
        .handler(handle_reopened())
        .handler(handle_resolved())
        .handler(handle_marked_read_and_reply())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<Command<'a, E>>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .no_safety_check()
        .load::<Contact>(&id)
        .data(pool.clone())
        .execute_all(executor)
        .await?
        .map(|loaded| Command::new(id, loaded, executor)))
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<CommandData, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::handler]
async fn handle_form_submitted<E: Executor>(
    _event: Event<FormSubmitted>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.status = Status::Unread;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}

#[evento::handler]
async fn handle_marked_read_and_reply<E: Executor>(
    _event: Event<MarkedReadAndReply>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.status = Status::Read;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}

#[evento::handler]
async fn handle_resolved<E: Executor>(
    _event: Event<Resolved>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.status = Status::Resolved;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}

#[evento::handler]
async fn handle_reopened<E: Executor>(
    _event: Event<Reopened>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.status = Status::Read;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}
