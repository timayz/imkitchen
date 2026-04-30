use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::recipe::rating::Viewed;

impl<E: Executor + Clone> super::Module<E> {
    pub async fn view(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        let user_id = user_id.into();
        let rating = self.load(&id, &user_id).await?;
        if !rating.viewed {
            rating
                .aggregator()?
                .event(&Viewed { recipe_id: id })
                .requested_by(user_id)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
