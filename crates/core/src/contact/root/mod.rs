use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregator, metadata::Event};
use imkitchen_types::contact::{
    self, FormSubmitted, MarkedReadAndReply, Reopened, Resolved, Status,
};
use std::ops::Deref;

mod mark_read_and_reply;
mod reopen;
mod resolve;
mod submit_form;

pub use submit_form::SubmitFormInput;

#[derive(Clone)]
pub struct Module<E: Executor>(crate::State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = crate::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    pub fn new(state: crate::State<E>) -> Self {
        Self(state)
    }

    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Contact>> {
        create_projection(id).execute(&self.executor).await
    }
}

#[evento::projection(Encode, Decode)]
pub struct Contact {
    pub id: String,
    pub status: Status,
}

impl ProjectionAggregator for Contact {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, Contact> {
    Projection::new::<contact::Contact>(id)
        .handler(handle_form_submitted())
        .handler(handle_reopened())
        .handler(handle_resolved())
        .handler(handle_marked_read_and_reply())
        .safety_check()
}

#[evento::handler]
async fn handle_form_submitted(
    event: Event<FormSubmitted>,
    row: &mut Contact,
) -> anyhow::Result<()> {
    row.id = event.aggregator_id.to_owned();
    row.status = Status::Unread;

    Ok(())
}

#[evento::handler]
async fn handle_marked_read_and_reply(
    _event: Event<MarkedReadAndReply>,
    row: &mut Contact,
) -> anyhow::Result<()> {
    row.status = Status::Read;

    Ok(())
}

#[evento::handler]
async fn handle_resolved(_event: Event<Resolved>, row: &mut Contact) -> anyhow::Result<()> {
    row.status = Status::Resolved;

    Ok(())
}

#[evento::handler]
async fn handle_reopened(_event: Event<Reopened>, row: &mut Contact) -> anyhow::Result<()> {
    row.status = Status::Read;

    Ok(())
}
