use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::favorite::Saved;

impl<E: Executor + Clone> super::Command<E> {
    pub async fn save(
        &self,
        id: impl Into<String>,
        owner_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        let user_id = user_id.into();
        let favorite = self.load(&id, &user_id).await?;

        if !favorite.saved {
            favorite
                .aggregator()?
                .event(&Saved {
                    recipe_id: id,
                    recipe_owner: owner_id.into(),
                })
                .requested_by(user_id)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
