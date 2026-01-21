use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::favorite::Unsaved;

impl<E: Executor + Clone> super::Command<E> {
    pub async fn unsave(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        let user_id = user_id.into();
        let favorite = self.load(&id, &user_id).await?;
        if favorite.saved {
            favorite
                .aggregator()?
                .event(&Unsaved { recipe_id: id })
                .requested_by(user_id)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
