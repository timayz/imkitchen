use evento::{Executor, ProjectionAggregator};
use imkitchen_types::recipe::SharedToCommunity;

impl<E: Executor + Clone> super::Module<E> {
    pub async fn share_to_community(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
        owner_name: impl Into<String>,
    ) -> crate::Result<()> {
        let Some(recipe) = self.load(id).await? else {
            crate::not_found!("recipe");
        };

        let request_by = request_by.into();
        if recipe.owner_id != request_by {
            crate::forbidden!("not owner of recipe");
        }

        if !recipe.is_shared {
            recipe
                .aggregator()?
                .event(&SharedToCommunity {
                    owner_name: owner_name.into(),
                })
                .requested_by(request_by)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
