use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::user::{Activated, State};

impl<E: Executor> super::Command<E> {
    pub async fn activate(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_shared::not_found!("user");
        };

        if user.state == State::Active {
            return Ok(());
        }

        user.aggregator()?
            .event(&Activated)
            .metadata(&Metadata::new(request_by))
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
