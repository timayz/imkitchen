use evento::{Executor, ProjectionAggregator};
use imkitchen_types::contact::{Reopened, Status};

impl<E: Executor + Clone> super::Module<E> {
    pub async fn reopen(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let Some(contact) = self.load(id).await? else {
            crate::not_found!("contact in mark_read_and_reply");
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
}
