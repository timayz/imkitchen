use std::ops::Deref;

use evento::{
    Executor, SkipEventData,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_shared::user::Registered;

pub mod admin;
pub mod global_stat;
pub mod login;

#[derive(Clone)]
pub struct Query<E: Executor>(pub imkitchen_shared::State<E>);

impl<E: Executor> Deref for Query<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn query_subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("user-query")
        .handler(handle_user_all())
        .safety_check()
}

#[evento::sub_all_handler]
async fn handle_user_all<E: Executor>(
    context: &Context<'_, E>,
    event: SkipEventData<Registered>,
) -> anyhow::Result<()> {
    admin::load(context.executor, &event.aggregator_id).await?;
    Ok(())
}
