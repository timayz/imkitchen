use std::ops::Deref;

use evento::{
    Executor, SkipEventData,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_shared::contact::FormSubmitted;
use sqlx::SqlitePool;

pub mod admin;
pub mod global_stat;

#[derive(Clone)]
pub struct Query<E: Executor>(pub imkitchen_shared::State<E>);

impl<E: Executor> Deref for Query<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn query_subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("contact-query")
        .handler(handle_contact_all())
        .safety_check()
}

#[evento::sub_all_handler]
async fn handle_contact_all<E: Executor>(
    context: &Context<'_, E>,
    event: SkipEventData<FormSubmitted>,
) -> anyhow::Result<()> {
    let (r, w) = context.extract::<(SqlitePool, SqlitePool)>();
    admin::load(context.executor, &r, &w, &event.aggregator_id).await?;
    Ok(())
}
