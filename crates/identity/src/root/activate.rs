use evento::{Executor, ProjectionAggregator};
use crate::types::user::{Activated, State};

impl<E: Executor> super::Module<E> {
    pub async fn activate(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_core::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_core::not_found!("user");
        };

        if user.state == State::Active {
            return Ok(());
        }

        user.aggregator()?
            .event(&Activated)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
