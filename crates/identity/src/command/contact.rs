use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregator, metadata::Event};
use imkitchen_shared::contact::{
    self, FormSubmitted, MarkedReadAndReply, Reopened, Resolved, Status,
};
use std::ops::Deref;

#[derive(Clone)]
pub struct Command<E: Executor>(imkitchen_shared::State<E>);

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Command<E> {
    pub fn new(state: imkitchen_shared::State<E>) -> Self {
        Self(state)
    }

    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Contact>> {
        create_projection(id).execute(&self.executor).await
    }

    pub async fn mark_read_and_reply(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let Some(contact) = self.load(id).await? else {
            imkitchen_shared::not_found!("contact in mark_read_and_reply");
        };

        if contact.status == Status::Read {
            return Ok(());
        }

        contact
            .aggregator()?
            .event(&MarkedReadAndReply)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    pub async fn reopen(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let Some(contact) = self.load(id).await? else {
            imkitchen_shared::not_found!("contact in mark_read_and_reply");
        };

        if contact.status == Status::Read {
            return Ok(());
        }

        contact
            .aggregator()?
            .event(&Reopened)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    pub async fn resolve(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let Some(contact) = self.load(id).await? else {
            imkitchen_shared::not_found!("contact in mark_read_and_reply");
        };

        if contact.status == Status::Resolved {
            return Ok(());
        }

        contact
            .aggregator()?
            .event(&Resolved)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}

#[derive(Validate)]
pub struct SubmitFormInput {
    #[validate(email)]
    pub to: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 25))]
    pub name: String,
    pub subject: Subject,
    #[validate(length(min = 1, max = 2000))]
    pub message: String,
}

impl<E: Executor + Clone> super::Command<E> {
    pub async fn submit_form(&self, input: SubmitFormInput) -> imkitchen_shared::Result<String> {
        input.validate()?;

        Ok(evento::create()
            .event(&FormSubmitted {
                to: input.to,
                name: input.name,
                email: input.email,
                subject: input.subject,
                message: input.message,
            })
            .commit(&self.executor)
            .await?)
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
