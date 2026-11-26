//! What moves the tutorials forward: the commands the users really make.
//!
//! No command knows of the tutorial. A subscription follows the events
//! they write and notes the step each one does, on the tutorial of the
//! user who made it: a moment after the fact. A step is noted once, so
//! delivery may repeat, and every node may run it.

use evento::metadata::{Event, RawEvent};
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::groceries::Groceries;
use imkitchen_aggregates::settings::Settings;
use imkitchen_aggregates::tutorial::Step;
use imkitchen_aggregates::{plat, recipe};

use super::model;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-api/tutorial";

/// Follows what the users do into their tutorials. It writes: start it
/// over a handle whose reads wait for the writes before them.
pub fn subscription<E: evento::Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .handler(on_recipe_created())
        .handler(on_plat_created())
        .handler(on_settings_event())
        .handler(on_list_generated())
        .handler(on_list_entry_added())
        .handler(on_groceries_event())
        .retry(5)
}

/// Written by hand or imported: both are `Created`.
#[evento::subscription]
async fn on_recipe_created<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<recipe::Created>,
) -> anyhow::Result<()> {
    let owner_id = &event.data.owner_id;
    model::complete(context.executor, owner_id, Step::Recipe, event.timestamp).await
}

#[evento::subscription]
async fn on_plat_created<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<plat::Created>,
) -> anyhow::Result<()> {
    let owner_id = &event.data.owner_id;
    model::complete(context.executor, owner_id, Step::Plat, event.timestamp).await
}

/// Any setting: the stream is under the id of its user.
#[evento::subscription_all]
async fn on_settings_event<E: evento::Executor>(
    context: &Context<'_, E>,
    event: RawEvent<Settings>,
) -> anyhow::Result<()> {
    let user_id = &event.aggregate_id;
    model::complete(context.executor, user_id, Step::Settings, event.timestamp).await
}

#[evento::subscription]
async fn on_list_generated<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_aggregates::list::Generated>,
) -> anyhow::Result<()> {
    let user_id = &event.aggregate_id;
    model::complete(context.executor, user_id, Step::List, event.timestamp).await
}

/// A meal picked by hand. One replaced or taken off is not what the step
/// asks.
#[evento::subscription]
async fn on_list_entry_added<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_aggregates::list::EntryAdded>,
) -> anyhow::Result<()> {
    let user_id = &event.aggregate_id;
    model::complete(context.executor, user_id, Step::List, event.timestamp).await
}

/// An item checked, unchecked or adjusted: the user is at their groceries.
#[evento::subscription_all]
async fn on_groceries_event<E: evento::Executor>(
    context: &Context<'_, E>,
    event: RawEvent<Groceries>,
) -> anyhow::Result<()> {
    let user_id = &event.aggregate_id;
    model::complete(context.executor, user_id, Step::Groceries, event.timestamp).await
}
