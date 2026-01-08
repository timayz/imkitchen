use evento::{Executor, metadata::Metadata};
use imkitchen_shared::user::{State, Suspended};

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn suspend(&self, request_by: impl Into<String>) -> imkitchen_shared::Result<()> {
        if self.state == State::Suspended {
            return Ok(());
        }

        self.aggregator()
            .event(&Suspended)
            .metadata(&Metadata::new(request_by))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
