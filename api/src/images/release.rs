//! Gives back the pictures the recipes let go of.
//!
//! A recipe names the use it makes of a picture; given another picture,
//! shown without one or deleted, it lets go of that use, and this
//! subscription releases it on the picture's stream. The last use released
//! takes the files with it: a picture two recipes show survives the
//! deletion of one of them.
//!
//! The handler does not act on the event it is given but on the recipe as
//! its stream leaves it: every use the recipe ever let go of is released,
//! which changes nothing for those released already. So delivery may
//! repeat, and every node runs the subscription at its own pace, a node
//! that was away long after the fact.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::recipe::{Deleted, ImageRemoved, ImageSet};

use super::model;
use crate::recipes;

/// The cursor identity of the subscription, in the store. One per node.
pub const SUBSCRIPTION: &str = "imkitchen-api/image-release";

pub fn subscription<E: evento::Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .handler(on_image_set())
        .handler(on_image_removed())
        .handler(on_deleted())
        .retry(5)
}

#[evento::subscription]
async fn on_image_set<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<ImageSet>,
) -> anyhow::Result<()> {
    release_let_go(context.executor, &event.aggregate_id).await
}

#[evento::subscription]
async fn on_image_removed<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<ImageRemoved>,
) -> anyhow::Result<()> {
    release_let_go(context.executor, &event.aggregate_id).await
}

#[evento::subscription]
async fn on_deleted<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    release_let_go(context.executor, &event.aggregate_id).await
}

/// Releases every use the recipe `recipe_id` let go of.
async fn release_let_go<E: evento::Executor>(executor: &E, recipe_id: &str) -> anyhow::Result<()> {
    for image in recipes::images_let_go(executor, recipe_id).await? {
        model::release(executor, &image.image_id, &image.use_id).await?;
    }
    Ok(())
}
