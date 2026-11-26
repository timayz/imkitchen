//! What keeps the lists free of the meals that are gone: a recipe or a plat
//! that is deleted leaves the list of its owner.
//!
//! The list is under the id of its user, so there is no table to look it
//! up in: the deletion says whose list to look at. A meal put on the list
//! while it was being deleted comes by later; so every event of a list has
//! its meals looked at too. One subscription does both, in the order of
//! the events: whichever of the two comes last finds the other. Taking off
//! what is no longer there writes nothing, so delivery may repeat, and
//! every node may run it.

use evento::metadata::{Event, RawEvent};
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::list::List;
use imkitchen_aggregates::{plat, recipe};

use super::model;
use crate::{plats, recipes};

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-api/list";

/// Follows the deleted recipes and plats out of the lists. It writes:
/// start it over a handle whose reads wait for the writes before them.
pub fn subscription<E: evento::Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .handler(on_list_event())
        .handler(on_recipe_deleted())
        .handler(on_plat_deleted())
        .retry(5)
}

#[evento::subscription_all]
async fn on_list_event<E: evento::Executor>(
    context: &Context<'_, E>,
    event: RawEvent<List>,
) -> anyhow::Result<()> {
    model::sweep(context.executor, &event.aggregate_id).await
}

#[evento::subscription]
async fn on_recipe_deleted<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<recipe::Deleted>,
) -> anyhow::Result<()> {
    let owner = recipes::owner_of(context.executor, &event.aggregate_id).await?;
    match owner {
        Some(owner_id) => model::sweep(context.executor, &owner_id).await,
        None => Ok(()),
    }
}

#[evento::subscription]
async fn on_plat_deleted<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<plat::Deleted>,
) -> anyhow::Result<()> {
    let owner = plats::owner_of(context.executor, &event.aggregate_id).await?;
    match owner {
        Some(owner_id) => model::sweep(context.executor, &owner_id).await,
        None => Ok(()),
    }
}
