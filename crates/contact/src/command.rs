use std::fmt::Display;

use evento::{Executor, LoadResult};
use imkitchen_shared::Metadata;
use serde::Deserialize;
use sqlx::SqlitePool;
use validator::Validate;

use crate::{Contact, FormSubmitted, MarkedAsReadAndReplay, Reopened, Resolved};

#[derive(Debug, Deserialize)]
pub enum ContactSubject {
    GeneralInquiry,
    TechnicalSupport,
    BillingQuestion,
    FeatureRequest,
    BugReport,
    PartnershipOpportunity,
    Other,
}

impl Display for ContactSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize)]
pub enum ContactStatus {
    Unread,
    Read,
    Resolved,
}

impl Display for ContactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Validate)]
pub struct SubmitContactFormInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 25))]
    pub name: String,
    pub subject: ContactSubject,
    #[validate(length(min = 1, max = 2000))]
    pub message: String,
}

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
    ) -> Result<LoadResult<Contact>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn submit_contact_form(
        &self,
        input: SubmitContactFormInput,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<String> {
        input.validate()?;

        Ok(evento::create::<Contact>()
            .data(&FormSubmitted {
                name: input.name,
                email: input.email,
                subject: input.subject.to_string(),
                message: input.message,
                status: ContactStatus::Unread.to_string(),
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?)
    }

    pub async fn mark_as_read_and_replay(
        &self,
        id: impl Into<String>,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<()> {
        let contact = self.load(id).await?;
        if contact.item.status == ContactStatus::Read.to_string() {
            return Ok(());
        }
        evento::save_with(contact)
            .data(&MarkedAsReadAndReplay {
                status: ContactStatus::Read.to_string(),
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn resolve(
        &self,
        id: impl Into<String>,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<()> {
        let contact = self.load(id).await?;
        if contact.item.status == ContactStatus::Resolved.to_string() {
            return Ok(());
        }
        evento::save_with(contact)
            .data(&Resolved {
                status: ContactStatus::Resolved.to_string(),
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn reopen(
        &self,
        id: impl Into<String>,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<()> {
        let contact = self.load(id).await?;
        if contact.item.status == ContactStatus::Read.to_string() {
            return Ok(());
        }
        evento::save_with(contact)
            .data(&Reopened {
                status: ContactStatus::Read.to_string(),
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}
