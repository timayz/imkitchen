use evento::{Executor, Projection, Snapshot, metadata::Event};
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

pub fn create_projection(id: impl Into<String>) -> Projection<CommandData> {
    Projection::new::<Contact>(id)
        .handler(handle_form_submitted())
        .handler(handle_reopened())
        .handler(handle_resolved())
        .handler(handle_marked_read_and_reply())
        .safety_check()
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<Command<'a, E>>, anyhow::Error> {
    let id = id.into();

    let Some(row) = create_projection(&id)
        .data(pool.clone())
        .execute(executor)
        .await?
    else {
        return Ok(None);
    };

    Ok(Some(Command::new(
        id,
        row.get_cursor_version()?,
        row,
        executor,
    )))
}

#[evento::handler]
async fn handle_form_submitted(
    _event: Event<FormSubmitted>,
    row: &mut CommandData,
) -> anyhow::Result<()> {
    row.status = Status::Unread;

    Ok(())
}

#[evento::handler]
async fn handle_marked_read_and_reply(
    _event: Event<MarkedReadAndReply>,
    row: &mut CommandData,
) -> anyhow::Result<()> {
    row.status = Status::Read;

    Ok(())
}

#[evento::handler]
async fn handle_resolved(_event: Event<Resolved>, row: &mut CommandData) -> anyhow::Result<()> {
    row.status = Status::Resolved;

    Ok(())
}

#[evento::handler]
async fn handle_reopened(_event: Event<Reopened>, row: &mut CommandData) -> anyhow::Result<()> {
    row.status = Status::Read;

    Ok(())
}
