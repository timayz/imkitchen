use evento::{Executor, metadata::Metadata};
use imkitchen_shared::contact::{Reopened, Status};

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn reopen(&self, request_by: impl Into<String>) -> imkitchen_shared::Result<()> {
        if self.status == Status::Read {
            return Ok(());
        }
        self.aggregator()
            .event(&Reopened)
            .metadata(&Metadata::new(request_by))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
