use crate::types::subscription::Cancelled;
use evento::{Executor, ProjectionAggregator};

impl<E: Executor> super::Module<E> {
    pub async fn cancel(&self, request_by: impl Into<String>) -> imkitchen_core::Result<()> {
        let request_by = request_by.into();
        let subscription = self.load(&request_by).await?;

        if subscription.is_active {
            subscription
                .aggregator()?
                .event(&Cancelled)
                .requested_by(request_by)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
