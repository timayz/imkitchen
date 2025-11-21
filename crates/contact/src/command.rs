use evento::{Executor, LoadResult};
use imkitchen_shared::Metadata;
use sqlx::SqlitePool;
use validator::Validate;

use crate::{Contact, FormSubmitted, MarkedReadAndReply, Reopened, Resolved};

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
    ) -> Result<LoadResult<Contact>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn load_optional(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<Contact>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }
}

#[derive(Validate)]
pub struct SubmitContactFormInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 25))]
    pub name: String,
    pub subject: Subject,
    #[validate(length(min = 1, max = 2000))]
    pub message: String,
}

impl<E: Executor + Clone> Command<E> {
    pub async fn submit_contact_form(
        &self,
        input: SubmitContactFormInput,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<String> {
        input.validate()?;

        Ok(evento::create::<Contact>()
            .data(&FormSubmitted {
                name: input.name,
                email: input.email,
                subject: input.subject,
                message: input.message,
                status: Status::Unread,
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?)
    }
}

impl<E: Executor + Clone> Command<E> {
    pub async fn mark_read_and_reply(
        &self,
        id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let contact = self.load(id).await?;
        if contact.item.status == Status::Read {
            return Ok(());
        }
        evento::save_with(contact)
            .data(&MarkedReadAndReply {
                status: Status::Read,
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}

impl<E: Executor + Clone> Command<E> {
    pub async fn resolve(
        &self,
        id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let contact = self.load(id).await?;
        if contact.item.status == Status::Resolved {
            return Ok(());
        }
        evento::save_with(contact)
            .data(&Resolved {
                status: Status::Resolved,
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}

impl<E: Executor + Clone> Command<E> {
    pub async fn reopen(
        &self,
        id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let contact = self.load(id).await?;
        if contact.item.status == Status::Read {
            return Ok(());
        }
        evento::save_with(contact)
            .data(&Reopened {
                status: Status::Read,
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}
