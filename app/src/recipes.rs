//! What app knows of the recipes: how many there are.
//!
//! A recipe is its owner's alone, and nobody signs in to app yet: until its
//! pages know who reads them, app keeps one row per recipe that is not
//! deleted, holding its id and nothing else, for the home to say how many
//! imkitchen keeps.
//!
//! The handlers leave the table as it should be when an event is delivered
//! twice (the subscription is at-least-once): a recipe is counted once
//! however often it is created, and gone however often it is deleted.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::recipe;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/recipes";

/// Follows the recipes into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_recipe_created())
        .handler(on_recipe_deleted())
}

#[evento::subscription]
async fn on_recipe_created<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<recipe::Created>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    db.execute(
        "INSERT INTO recipes (id) VALUES (?1) ON CONFLICT (id) DO NOTHING",
        (event.aggregate_id.as_str(),),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_recipe_deleted<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<recipe::Deleted>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    db.execute(
        "DELETE FROM recipes WHERE id = ?1",
        (event.aggregate_id.as_str(),),
    )
    .await?;
    Ok(())
}

/// How many recipes there are, as far as app has followed.
pub async fn count(db: &Db) -> anyhow::Result<u64> {
    let count: i64 = db.fetch_scalar("SELECT count(*) FROM recipes", ()).await?;
    Ok(count.max(0) as u64)
}
