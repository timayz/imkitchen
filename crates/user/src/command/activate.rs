use evento::{Executor, metadata::Metadata};

use crate::{Activated, State};

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn activate(&self) -> imkitchen_shared::Result<()> {
        if self.state == State::Active {
            return Ok(());
        }

        self.aggregator()
            .event(&Activated)
            .metadata(&Metadata::default())
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
