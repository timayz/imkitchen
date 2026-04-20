use evento::{
    Aggregator, AggregatorEvent, Executor, ReadAggregator,
    cursor::Args,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_shared::user::{
    invoice::{Created, InvoiceAddress},
    subscription::{Address, StripePaymentIntentSucceeded},
};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::query::invoice_user;

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("user-invoice").handler(handler_payment_intent_succeeded())
}

#[evento::subscription]
async fn handler_payment_intent_succeeded<E: Executor>(
    context: &Context<'_, E>,
    event: Event<StripePaymentIntentSucceeded>,
) -> anyhow::Result<()> {
    let dt = OffsetDateTime::from_unix_timestamp(event.timestamp as i64)?;
    let fmt = time::macros::format_description!("[year]-[month]");
    let key = dt.format(&fmt)?;

    let result = context
        .executor
        .read(
            Some(vec![ReadAggregator::event(
                Created::aggregator_type(),
                Created::event_name(),
            )]),
            None,
            Args::backward(1, None),
        )
        .await?;

    let number = match result.edges.first() {
        Some(e) => {
            let data: Created = bitcode::decode(&e.node.data)?;

            if data.key == key { data.number + 1 } else { 1 }
        }
        _ => 1,
    };

    let id = evento::create()
        .metadata_from(&event.metadata)
        .event(&Created {
            key,
            number,
            from: InvoiceAddress {
                name: "timada".to_owned(),
                email: "support@imkitchen.app".to_owned(),
                address: Address {
                    city: Some("Paris".to_owned()),
                    country: Some("FR".to_owned()),
                    line1: Some("60 RUE FRANCOIS IER".to_owned()),
                    line2: None,
                    postal_code: Some("75008".to_owned()),
                    state: None,
                },
            },
            to: InvoiceAddress {
                name: event.data.name,
                email: event.data.email,
                address: event.data.address,
            },
            payment_method_id: event.data.payment_method_id,
            details: event.data.details,
            paid_at: event.data.paid_at,
            expire_at: event.data.expire_at,
        })
        .commit(context.executor)
        .await?;

    let (r, w) = context.extract::<(SqlitePool, SqlitePool)>();
    invoice_user::load(context.executor, &r, &w, &id).await?;
    Ok(())
}
