use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::contact::{MarkedReadAndReply, Status};

impl<E: Executor + Clone> super::Command<E> {
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
}
