use crate::types::user::{Role, RoleChanged};
use evento::{Executor, ProjectionAggregator};

impl<E: Executor> super::Module<E> {
    pub async fn change_role(
        &self,
        id: impl Into<String>,
        role: Role,
        request_by: impl Into<String>,
    ) -> imkitchen_core::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_core::not_found!("user");
        };

        if user.role == role {
            return Ok(());
        }

        user.aggregator()?
            .event(&RoleChanged { role })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
