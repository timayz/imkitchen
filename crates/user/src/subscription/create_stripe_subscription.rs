use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::user::subscription::StripeSubscriptionCreated;

impl<E: Executor> super::Command<E> {
    pub async fn create_stripe_subscription(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        let subscription = self.load(&request_by).await?;

        subscription
            .aggregator()?
            .event(&StripeSubscriptionCreated { id: id.into() })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
