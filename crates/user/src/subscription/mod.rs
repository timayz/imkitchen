mod create_stripe_customer;
mod create_stripe_subscription;
mod toogle_life_premium;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, metadata::Event};
use imkitchen_shared::user::subscription;
use std::ops::Deref;

#[derive(Clone)]
pub struct Command<E: Executor>(pub(crate) imkitchen_shared::State<E>);

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Command<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Subscription> {
        let id = id.into();
        create_projection::<E>(&id)
            .execute(&self.executor)
            .await
            .map(|r| {
                r.unwrap_or_else(|| Subscription {
                    id,
                    expire_at: 0,
                    cursor: Default::default(),
                    customer_id: None,
                    subscription_id: None,
                })
            })
    }
}

#[evento::projection(Encode, Decode)]
pub struct Subscription {
    pub id: String,
    pub customer_id: Option<String>,
    pub subscription_id: Option<String>,
    pub expire_at: u64,
}

fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, Subscription> {
    Projection::new::<subscription::Subscription>(id)
        .handler(handle_life_premium_toggled())
        .handler(handle_stripe_customer_created())
        .handler(handle_stripe_subscription_created())
        .safety_check()
}

impl evento::ProjectionAggregator for Subscription {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_life_premium_toggled(
    event: Event<subscription::LifePremiumToggled>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.expire_at = event.data.expire_at;

    Ok(())
}

#[evento::handler]
async fn handle_stripe_customer_created(
    event: Event<subscription::StripeCustomerCreated>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.customer_id = Some(event.data.id);

    Ok(())
}

#[evento::handler]
async fn handle_stripe_subscription_created(
    event: Event<subscription::StripeSubscriptionCreated>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.subscription_id = Some(event.data.id);

    Ok(())
}
