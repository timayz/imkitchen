//! The read side of the plats: the `plats` table a user's list is read
//! from, and the subscription that keeps it current and the plats free of
//! the recipes that were deleted.
//!
//! A handler reloads the plat from its stream and rewrites its row, so an
//! event delivered twice (the subscription is at-least-once) leaves the row
//! as it should be; a deleted plat loses its row.
//!
//! A recipe that is deleted is taken out of the plats it was part of
//! (`RecipeRemoved`, on each of them): those the table knows of when the
//! deletion comes by. A plat composed with the recipe while it was being
//! deleted comes by later; so every event of a plat has its recipes looked
//! at, and those that are gone taken out. One subscription does both, in
//! the order of the events: whichever of the two comes last finds the
//! other. Taking out what is no longer there writes nothing, so delivery
//! may repeat, and every node runs the subscription over its own table,
//! with a cursor of its own.

use evento::metadata::{Event, RawEvent};
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::plat::Plat;
use imkitchen_aggregates::recipe;
use timada_core::db::{FromRow, Json, search_key};
use timada_core::like_contains;
use timada_core::page::Cursor;

use super::model::{self, PlatState};
use crate::db::Db;
use crate::recipes;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-api/plats";

/// Follows the plats into `db`, and the deleted recipes out of the plats.
/// It writes: start it over a handle whose reads wait for the writes
/// before them.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_plat_event())
        .handler(on_recipe_deleted())
        .retry(5)
}

#[evento::subscription_all]
async fn on_plat_event<E: evento::Executor>(
    context: &Context<'_, E>,
    event: RawEvent<Plat>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let Some(plat) = model::load(context.executor, &event.aggregate_id).await? else {
        return Ok(());
    };
    if plat.deleted {
        db.execute("DELETE FROM plats WHERE id = ?1", (plat.id.as_str(),))
            .await?;
        return Ok(());
    }
    upsert(&db, &plat).await?;

    // Composed with a recipe that was being deleted: its deletion came by
    // before the plat was there to take it out of.
    let mut gone = Vec::new();
    for id in &plat.composition.recipe_ids {
        if recipes::own(context.executor, &plat.owner_id, id)
            .await?
            .is_none()
        {
            gone.push(id.to_owned());
        }
    }
    model::remove_recipes(context.executor, &plat.id, &gone).await
}

#[evento::subscription]
async fn on_recipe_deleted<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<recipe::Deleted>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let recipe_id = &event.aggregate_id;
    let Some(owner_id) = recipes::owner_of(context.executor, recipe_id).await? else {
        return Ok(());
    };
    let gone = [recipe_id.to_owned()];
    // A user has a few dozen plats: all of theirs are looked at.
    let plats: Vec<Composed> = db
        .fetch_all(
            "SELECT id, recipe_ids FROM plats WHERE owner_id = ?1",
            (owner_id.as_str(),),
        )
        .await?;
    for plat in plats {
        if plat.recipe_ids.0.contains(recipe_id) {
            model::remove_recipes(context.executor, &plat.id, &gone).await?;
        }
    }
    Ok(())
}

/// What a plat is made of, as the table knows it.
#[derive(FromRow)]
struct Composed {
    id: String,
    recipe_ids: Json<Vec<String>>,
}

/// Writes the row of a plat as its stream leaves it.
async fn upsert(db: &Db, plat: &PlatState) -> anyhow::Result<()> {
    let composition = &plat.composition;
    db.execute(
        "INSERT INTO plats (id, owner_id, name, recipe_ids, search, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT (id) DO UPDATE SET
            name = excluded.name, recipe_ids = excluded.recipe_ids,
            search = excluded.search, updated_at = excluded.updated_at",
        (
            plat.id.as_str(),
            plat.owner_id.as_str(),
            composition.name.as_str(),
            Json(&composition.recipe_ids).to_text()?,
            search_key(&composition.name),
            plat.created_at as i64,
            plat.updated_at as i64,
        ),
    )
    .await?;
    Ok(())
}

/// One row of `plats`: what a list shows of a plat, but for its recipes,
/// which it names.
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct PlatRow {
    pub id: String,
    pub name: String,
    /// The recipes it is made of, in the order they are served.
    pub recipe_ids: Json<Vec<String>>,
    /// Unix seconds.
    pub created_at: i64,
    /// Unix seconds.
    pub updated_at: i64,
}

impl PlatRow {
    /// Where the row stands in its owner's list.
    pub fn cursor(&self) -> Cursor {
        Cursor {
            at: self.created_at,
            id: self.id.clone(),
        }
    }
}

/// The plats of `owner_id`, the most recent first, `limit` at most: those
/// after `after` in the list, and only those `q` is found in the name of
/// (whatever the case and the accents) when there is one.
pub async fn plats_of(
    db: &Db,
    owner_id: &str,
    q: Option<&str>,
    after: Option<&Cursor>,
    limit: i64,
) -> anyhow::Result<Vec<PlatRow>> {
    let (at, id) = after.map_or((i64::MAX, ""), |cursor| (cursor.at, cursor.id.as_str()));
    db.fetch_all(
        "SELECT id, name, recipe_ids, created_at, updated_at
         FROM plats INDEXED BY plats_listed
         WHERE owner_id = ?1
            AND created_at <= ?2 AND (created_at < ?2 OR id < ?3)
            AND (?4 IS NULL OR search LIKE ?4 ESCAPE '\\')
         ORDER BY created_at DESC, id DESC LIMIT ?5",
        (
            owner_id,
            at,
            id,
            q.map(|q| like_contains(&search_key(q))),
            limit,
        ),
    )
    .await
}
