use evento::{Executor, ProjectionAggregator};
use crate::types::user::{State, Suspended};

impl<E: Executor> super::Module<E> {
    pub async fn suspend(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_core::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_core::not_found!("user");
        };

        if user.state == State::Suspended {
            return Ok(());
        }

        user.aggregator()?
            .event(&Suspended)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
