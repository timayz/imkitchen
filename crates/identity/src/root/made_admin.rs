use crate::types::user::{MadeAdmin, Role};
use evento::{Executor, ProjectionAggregate};

impl<E: Executor> super::Module<E> {
    pub async fn made_admin(&self, id: impl Into<String>) -> imkitchen_core::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_core::not_found!("user");
        };

        if user.role == Role::Admin {
            return Ok(());
        }

        user.write()?
            .event(&MadeAdmin)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
