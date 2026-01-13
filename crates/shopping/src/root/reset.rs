use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::shopping::Resetted;

impl<E: Executor> super::Command<E> {
    pub async fn reset(
        &self,
        week: u64,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        let Some(shopping) = self.load(&request_by).await? else {
            imkitchen_shared::not_found!("shopping in reset");
        };

        shopping
            .aggregator()?
            .event(&Resetted { week })
            .metadata(&Metadata::new(request_by))
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
