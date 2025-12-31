use evento::{Executor, metadata::Metadata};

use crate::SharedToCommunity;

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn share_to_community(
        &self,
        request_by: impl Into<String>,
        owner_name: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        if self.is_deleted {
            imkitchen_shared::not_found!("recipe");
        }

        let request_by = request_by.into();
        if self.owner_id != request_by {
            imkitchen_shared::forbidden!("not owner of recipe");
        }

        if !self.is_shared {
            self.aggregator()
                .event(&SharedToCommunity {
                    owner_name: owner_name.into(),
                })
                .metadata(&Metadata::new(request_by))
                .commit(self.executor)
                .await?;
        }

        Ok(())
    }
}
