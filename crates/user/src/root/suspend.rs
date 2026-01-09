use evento::{Executor, ProjectionCursor, metadata::Metadata};
use imkitchen_shared::user::{State, Suspended};

impl<E: Executor> super::Command<E> {
    pub async fn suspend(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_shared::not_found!("user");
        };

        if user.state == State::Suspended {
            return Ok(());
        }

        user.aggregator()?
            .event(&Suspended)
            .metadata(&Metadata::new(request_by))
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
