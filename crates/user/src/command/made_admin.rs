use evento::{Executor, metadata::Metadata};

use crate::{MadeAdmin, Role};

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn made_admin(&self) -> imkitchen_shared::Result<()> {
        if self.role == Role::Admin {
            return Ok(());
        }

        self.aggregator()
            .event(&MadeAdmin)
            .metadata(&Metadata::default())
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
