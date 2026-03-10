use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::user::subscription::StripePaymentIntentSucceeded;
use stripe_shared::{PaymentIntent, PaymentIntentStatus};

impl<E: Executor> super::Command<E> {
    pub async fn update_stripe_payment_intent_status(
        &self,
        intent: impl Into<PaymentIntent>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        let subscription = self.load(&request_by).await?;
        let intent = intent.into();

        match (
            intent.status,
            intent.payment_method,
            intent.metadata.get("plan"),
            intent.metadata.get("country"),
            intent.metadata.get("state"),
        ) {
            (
                PaymentIntentStatus::Succeeded,
                Some(method),
                Some(plan),
                Some(country),
                Some(state),
            ) => {
                let months = match plan.as_str() {
                    "monthly" => 1,
                    "annual" => 12,
                    plan => imkitchen_shared::server!("unrecognized subscription plan {plan}"),
                };

                let expire_at = super::add_months(intent.created, months);

                subscription
                    .aggregator()?
                    .event(&StripePaymentIntentSucceeded {
                        id: intent.id.to_string(),
                        payment_method_id: method.id().to_string(),
                        plan: plan.to_owned(),
                        country: country.to_owned(),
                        state: state.to_owned(),
                        expire_at: expire_at.try_into()?,
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
