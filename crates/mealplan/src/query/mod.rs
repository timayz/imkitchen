pub mod last_week;
pub mod slot;
pub mod week;

use std::ops::Deref;

use evento::{
    Executor,
    metadata::RawEvent,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_shared::mealplan::WeekGenerated;

#[derive(Clone)]
pub struct Query<E: Executor>(pub imkitchen_shared::State<E>);

impl<E: Executor> Deref for Query<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn query_subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("mealplan-query")
        .handler(handle_mealplan_all())
        .safety_check()
}

#[evento::subscription_all]
async fn handle_mealplan_all<E: Executor>(
    context: &Context<'_, E>,
    event: RawEvent<WeekGenerated>,
) -> anyhow::Result<()> {
    last_week::load(context.executor, &event.aggregator_id).await?;
    Ok(())
}
