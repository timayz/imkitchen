use evento::{Executor, metadata::Metadata};

use imkitchen_shared::contact::{MarkedReadAndReply, Status};

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn mark_read_and_reply(
        &self,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        if self.status == Status::Read {
            return Ok(());
        }
        self.aggregator()
            .event(&MarkedReadAndReply)
            .metadata(&Metadata::new(request_by))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
