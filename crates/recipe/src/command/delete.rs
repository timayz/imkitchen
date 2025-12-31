use evento::{Executor, metadata::Metadata};

use crate::Deleted;

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn delete(&self, request_by: impl Into<String>) -> imkitchen_shared::Result<()> {
        if self.is_deleted {
            imkitchen_shared::not_found!("recipe not found");
        }

        let request_by = request_by.into();
        if self.owner_id != request_by {
            imkitchen_shared::forbidden!("not owner of recipe");
        }

        self.aggregator()
            .event(&Deleted)
            .metadata(&Metadata::new(request_by))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}
