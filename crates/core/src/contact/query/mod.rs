use evento::{
    Executor,
    metadata::RawEvent,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_types::contact::FormSubmitted;
use sqlx::SqlitePool;

pub mod admin;
pub mod global_stat;

pub fn query_subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("contact-query")
        .handler(handle_contact_all())
        .safety_check()
}

#[evento::subscription_all]
async fn handle_contact_all<E: Executor>(
    context: &Context<'_, E>,
    event: RawEvent<FormSubmitted>,
) -> anyhow::Result<()> {
    let (r, w) = context.extract::<(SqlitePool, SqlitePool)>();
    admin::load(context.executor, &r, &w, &event.aggregator_id).await?;
    Ok(())
}
