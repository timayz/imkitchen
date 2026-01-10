pub mod user;
pub mod user_stat;

use std::ops::Deref;

use evento::{
    Executor, SkipEventData,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_shared::recipe::Created;

#[derive(Clone)]
pub struct Query<E: Executor>(pub imkitchen_shared::State<E>);

impl<E: Executor> Deref for Query<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn query_subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-query")
        .handler(handle_recipe_all())
        .safety_check()
}

#[evento::sub_all_handler]
async fn handle_recipe_all<E: Executor>(
    context: &Context<'_, E>,
    event: SkipEventData<Created>,
) -> anyhow::Result<()> {
    user::load(context.executor, &event.aggregator_id).await?;
    Ok(())
}
