use evento::{
    Aggregator, AggregatorEvent, Executor, ReadAggregator,
    cursor::Args,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_shared::user::{invoice::Created, subscription::StripePaymentIntentSucceeded};
use time::OffsetDateTime;

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

    let id = format!("{key}-{number:02}");

    evento::aggregator(id)
        .metadata_from(&event.metadata)
        .event(&Created {
            key,
            number,
            name: event.data.name,
            payment_method_id: event.data.payment_method_id,
            address: event.data.address,
            details: event.data.details,
            paid_at: event.data.paid_at,
            expire_at: event.data.expire_at,
        })
        .commit(context.executor)
        .await?;

    Ok(())
}
