use evento::{Executor, metadata::Metadata};
use imkitchen_shared::contact::{Resolved, Status};

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn resolve(&self, request_by: impl Into<String>) -> imkitchen_shared::Result<()> {
        if self.status == Status::Resolved {
            return Ok(());
        }
        self.aggregator()
            .event(&Resolved)
            .metadata(&Metadata::new(request_by))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
