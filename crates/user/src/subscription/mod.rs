mod cancel;
mod create_stripe_customer;
mod create_stripe_payment_intent;
mod create_stripe_setup_intent;
mod toogle_life_premium;
mod update_stripe_payment_intent_status;
mod update_stripe_setup_intent_status;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, metadata::Event};
use imkitchen_shared::user::subscription::{self, Address, PaymentDetails};
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
                    address: None,
                    expire_at: 0,
                    cursor: Default::default(),
                    customer_id: None,
                    payment_method_id: None,
                    payment_intent_id: None,
                    is_active: true,
                    payment_details: None,
                    setup_intent_id: None,
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
    pub setup_intent_id: Option<String>,
    pub payment_details: Option<PaymentDetails>,
    pub address: Option<Address>,
    pub expire_at: u64,
    pub is_active: bool,
}

fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, Subscription> {
    Projection::new::<subscription::Subscription>(id)
        .handler(handle_life_premium_toggled())
        .handler(handle_stripe_customer_created())
        .handler(handle_stripe_payment_intent_created())
        .handler(handle_stripe_payment_intent_succeeded())
        .handler(handle_cancelled())
        .handler(handle_stripe_setup_intent_created())
        .handler(handle_stripe_setup_intent_succeeded())
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
async fn handle_stripe_payment_intent_created(
    event: Event<subscription::StripePaymentIntentCreated>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.payment_intent_id = Some(event.data.id);
    data.payment_details = Some(event.data.details);

    Ok(())
}

#[evento::handler]
async fn handle_stripe_setup_intent_created(
    event: Event<subscription::StripeSetupIntentCreated>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.setup_intent_id = Some(event.data.id);

    Ok(())
}

#[evento::handler]
async fn handle_stripe_payment_intent_succeeded(
    event: Event<subscription::StripePaymentIntentSucceeded>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.payment_intent_id = None;
    data.payment_method_id = Some(event.data.payment_method_id);
    data.name = event.data.name;
    data.address = event.data.address;
    data.expire_at = event.data.expire_at;
    data.is_active = true;

    Ok(())
}

#[evento::handler]
async fn handle_stripe_setup_intent_succeeded(
    event: Event<subscription::StripeSetupIntentSucceeded>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.setup_intent_id = None;
    data.name = event.data.name;
    data.address = event.data.address;
    data.payment_method_id = Some(event.data.payment_method_id);

    Ok(())
}

#[evento::handler]
async fn handle_cancelled(
    _event: Event<subscription::Cancelled>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.is_active = false;
    data.payment_method_id = None;

    Ok(())
}

pub(super) fn add_months(timestamp: i64, months: u8) -> i64 {
    let dt = OffsetDateTime::from_unix_timestamp(timestamp).unwrap();

    let total_months = dt.month() as u8 + months;
    let year_offset = (total_months - 1) / 12;
    let new_month = Month::try_from((total_months - 1) % 12 + 1).unwrap();

    let new_year = dt.year() + year_offset as i32;

    let day = dt.day();

    if day > 28 {
        // Roll over to day 1 of the following month
        let total_months_next = new_month as u8 + 1;
        let year_offset_next = (total_months_next - 1) / 12;
        let next_month = Month::try_from((total_months_next - 1) % 12 + 1).unwrap();
        let next_year = new_year + year_offset_next as i32;

        dt.replace_year(next_year)
            .unwrap()
            .replace_month(next_month)
            .unwrap()
            .replace_day(1)
            .unwrap()
            .unix_timestamp()
    } else {
        dt.replace_year(new_year)
            .unwrap()
            .replace_month(new_month)
            .unwrap()
            .replace_day(day)
            .unwrap()
            .unix_timestamp()
    }
}
