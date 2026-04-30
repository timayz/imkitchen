use evento::{Executor, ProjectionAggregator};
use imkitchen_types::favorite::Unsaved;

impl<E: Executor + Clone> super::Module<E> {
    pub async fn unsave(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> crate::Result<()> {
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
