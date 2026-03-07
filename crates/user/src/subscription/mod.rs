mod create_stripe_customer;
mod create_stripe_payment_intent;
mod create_stripe_subscription;
mod toogle_life_premium;
mod update_stripe_payment_intent_status;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, metadata::Event};
use imkitchen_shared::user::subscription;
use std::ops::Deref;
use time::{Month, OffsetDateTime};

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
                    name: None,
                    expire_at: 0,
                    cursor: Default::default(),
                    customer_id: None,
                    payment_method_id: None,
                    payment_intent_id: None,
                    is_active: true,
                    plan: None,
                })
            })
    }
}

#[evento::projection(Encode, Decode)]
pub struct Subscription {
    pub id: String,
    pub name: Option<String>,
    pub customer_id: Option<String>,
    pub payment_method_id: Option<String>,
    pub payment_intent_id: Option<String>,
    pub plan: Option<String>,
    pub expire_at: u64,
    pub is_active: bool,
}

fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, Subscription> {
    Projection::new::<subscription::Subscription>(id)
        .revision(12)
        .handler(handle_life_premium_toggled())
        .handler(handle_stripe_customer_created())
        .handler(handle_stripe_payment_method_created())
        .handler(handle_stripe_payment_intent_created())
        .handler(handle_stripe_payment_intent_succeeded())
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
async fn handle_stripe_payment_method_created(
    event: Event<subscription::StripePaymentMethodCreated>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.payment_method_id = Some(event.data.id);

    Ok(())
}

#[evento::handler]
async fn handle_stripe_payment_intent_created(
    event: Event<subscription::StripePaymentIntentCreated>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.payment_intent_id = Some(event.data.id);

    Ok(())
}

#[evento::handler]
async fn handle_stripe_payment_intent_succeeded(
    event: Event<subscription::StripePaymentIntentSucceeded>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.payment_intent_id = Some(event.data.id);
    data.expire_at = event.data.expire_at;
    data.plan = Some(event.data.plan);
    data.is_active = true;

    Ok(())
}

pub(super) fn add_months(timestamp: i64, months: u8) -> i64 {
    let dt = OffsetDateTime::from_unix_timestamp(timestamp).unwrap();

    let total_months = dt.month() as u8 + months;
    let year_offset = (total_months - 1) / 12;
    let new_month = Month::try_from((total_months - 1) % 12 + 1).unwrap();

    let new_year = dt.year() + year_offset as i32;

    // Clamp day to last valid day of the new month
    let days_in_month = new_month.length(new_year);
    let clamped_day = dt.day().min(days_in_month);

    dt.replace_year(new_year)
        .unwrap()
        .replace_month(new_month)
        .unwrap()
        .replace_day(clamped_day)
        .unwrap()
        .unix_timestamp()
}
