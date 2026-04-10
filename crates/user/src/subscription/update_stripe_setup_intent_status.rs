use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::user::subscription::StripeSetupIntentSucceeded;
use stripe_shared::{SetupIntent, SetupIntentStatus};
use stripe_types::Expandable;

impl<E: Executor> super::Command<E> {
    pub async fn update_stripe_setup_intent_status(
        &self,
        intent: impl Into<SetupIntent>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        let subscription = self.load(&request_by).await?;
        let intent = intent.into();

        if let (SetupIntentStatus::Succeeded, Some(Expandable::Object(method))) =
            (intent.status, intent.payment_method)
        {
            subscription
                .aggregator()?
                .event(&StripeSetupIntentSucceeded {
                    id: intent.id.to_string(),
                    name: method.billing_details.name.to_owned(),
                    address: method.billing_details.address.map(|a| a.into()),
                    payment_method_id: method.id.to_string(),
                })
                .requested_by(request_by)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
