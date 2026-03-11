use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::user::subscription::StripeSetupIntentSucceeded;
use stripe_shared::{SetupIntent, SetupIntentStatus};

impl<E: Executor> super::Command<E> {
    pub async fn update_stripe_setup_intent_status(
        &self,
        intent: impl Into<SetupIntent>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        let subscription = self.load(&request_by).await?;
        let intent = intent.into();

        let Some(metadata) = intent.metadata else {
            imkitchen_shared::server!("setup intent metadata not defined");
        };

        match (
            intent.status,
            intent.payment_method,
            metadata.get("country"),
            metadata.get("state"),
        ) {
            (SetupIntentStatus::Succeeded, Some(method), Some(country), Some(state)) => {
                subscription
                    .aggregator()?
                    .event(&StripeSetupIntentSucceeded {
                        id: intent.id.to_string(),
                        payment_method_id: method.id().to_string(),
                        country: country.to_owned(),
                        state: state.to_owned(),
                    })
                    .requested_by(request_by)
                    .commit(&self.executor)
                    .await?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}
