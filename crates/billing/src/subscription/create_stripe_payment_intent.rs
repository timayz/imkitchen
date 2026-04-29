use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::user::subscription::{PaymentDetails, StripePaymentIntentCreated};

impl<E: Executor> super::Command<E> {
    pub async fn create_stripe_payment_intent(
        &self,
        id: impl Into<String>,
        email: impl Into<String>,
        request_by: impl Into<String>,
        details: PaymentDetails,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        let email = email.into();
        let subscription = self.load(&request_by).await?;

        subscription
            .aggregator()?
            .event(&StripePaymentIntentCreated {
                id: id.into(),
                email,
                details,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
