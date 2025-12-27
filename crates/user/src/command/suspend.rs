use evento::{Executor, metadata::Metadata};

use crate::{State, Suspended};

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn suspend(&self) -> imkitchen_shared::Result<()> {
        if self.state == State::Suspended {
            return Ok(());
        }

        self.aggregator()
            .event(&Suspended)
            .metadata(&Metadata::default())
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
