use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::contact::{Reopened, Status};

impl<E: Executor + Clone> super::Command<E> {
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
            .metadata(&Metadata::new(request_by))
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
