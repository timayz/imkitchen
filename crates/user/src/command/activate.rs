use evento::{Executor, metadata::Metadata};

use crate::{Activated, State};

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn activate(&self, request_by: impl Into<String>) -> imkitchen_shared::Result<()> {
        if self.state == State::Active {
            return Ok(());
        }

        self.aggregator()
            .event(&Activated)
            .metadata(&Metadata::new(request_by))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
