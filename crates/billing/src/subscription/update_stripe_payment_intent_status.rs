use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::user::subscription::StripePaymentIntentSucceeded;
use stripe_shared::{PaymentIntent, PaymentIntentStatus};
use stripe_types::Expandable;

impl<E: Executor> super::Command<E> {
    pub async fn update_stripe_payment_intent_status(
        &self,
        intent: impl Into<PaymentIntent>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        let subscription = self.load(&request_by).await?;
        let intent = intent.into();

        if let (PaymentIntentStatus::Succeeded, Some(Expandable::Object(method)), Some(details)) = (
            intent.status,
            intent.payment_method,
            subscription.payment_details.clone(),
        ) {
            let months = match details.plan.as_str() {
                "monthly" => 1,
                "annual" => 12,
                plan => imkitchen_shared::server!("unrecognized subscription plan {plan}"),
            };

            let expire_at = super::add_months(intent.created, months);

            let Some(name) = method.billing_details.name.to_owned() else {
                imkitchen_shared::user!("name is missing from payment intent");
            };

            let Some(email) = subscription.email.to_owned() else {
                imkitchen_shared::user!("email is missing from subscription");
            };

            let Some(address) = method.billing_details.address.to_owned() else {
                imkitchen_shared::user!("address is missing from payment intent");
            };

            subscription
                .aggregator()?
                .event(&StripePaymentIntentSucceeded {
                    id: intent.id.to_string(),
                    payment_method_id: method.id.to_string(),
                    name,
                    email,
                    address: address.into(),
                    details,
                    paid_at: intent.created.try_into()?,
                    expire_at: expire_at.try_into()?,
                })
                .requested_by(request_by)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
