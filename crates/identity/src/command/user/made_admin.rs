use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::user::{MadeAdmin, Role};

impl<E: Executor> super::Command<E> {
    pub async fn made_admin(&self, id: impl Into<String>) -> imkitchen_shared::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_shared::not_found!("user");
        };

        if user.role == Role::Admin {
            return Ok(());
        }

        user.aggregator()?
            .event(&MadeAdmin)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
