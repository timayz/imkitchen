//! What app knows of the dishes (the plats of the api): the name of each
//! one, whose it is and the recipes it is made of, in the order they are
//! served.
//!
//! A dish names its recipes and holds nothing of them: a page shows each
//! as the recipes of app know it ([`crate::recipes`]). Like them a dish is
//! its owner's alone: nothing is read from the table without saying whose
//! is asked for.
//!
//! app keeps one row per dish that is not deleted. The handlers leave the
//! table as it should be when an event is delivered twice (the subscription
//! is at-least-once): a dish is written once however often it is composed,
//! recomposed as its last event says, without a recipe however often the
//! recipe is taken out of it, and gone however often it is deleted.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::plat::{Created, Deleted, Edited, RecipeRemoved};
use timada_core::db::{FromRow, Json, search_key};
use timada_core::like_contains;
use timada_core::page::Cursor;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/dishes";

/// Follows the dishes into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_created())
        .handler(on_edited())
        .handler(on_deleted())
        .handler(on_recipe_removed())
}

#[evento::subscription]
async fn on_created<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let data = &event.data;
    db.execute(
        "INSERT INTO dishes (id, owner_id, name, recipe_ids, search, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
         ON CONFLICT (id) DO NOTHING",
        (
            event.aggregate_id.as_str(),
            data.owner_id.as_str(),
            data.name.as_str(),
            Json(&data.recipe_ids).to_text()?,
            search_key(&data.name),
            event.timestamp as i64,
        ),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_edited<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Edited>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let data = &event.data;
    db.execute(
        "UPDATE dishes SET name = ?2, recipe_ids = ?3, search = ?4, updated_at = ?5
         WHERE id = ?1",
        (
            event.aggregate_id.as_str(),
            data.name.as_str(),
            Json(&data.recipe_ids).to_text()?,
            search_key(&data.name),
            event.timestamp as i64,
        ),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_deleted<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    db.execute(
        "DELETE FROM dishes WHERE id = ?1",
        (event.aggregate_id.as_str(),),
    )
    .await?;
    Ok(())
}

/// What a dish is made of, as the table keeps it.
#[derive(FromRow)]
struct Composed {
    recipe_ids: Json<Vec<String>>,
}

#[evento::subscription]
async fn on_recipe_removed<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<RecipeRemoved>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let id = event.aggregate_id.as_str();
    let composed: Option<Composed> = db
        .fetch_optional("SELECT recipe_ids FROM dishes WHERE id = ?1", (id,))
        .await?;
    let Some(Composed {
        recipe_ids: Json(mut recipe_ids),
    }) = composed
    else {
        return Ok(());
    };
    recipe_ids.retain(|recipe_id| *recipe_id != event.data.recipe_id);
    db.execute(
        "UPDATE dishes SET recipe_ids = ?2, updated_at = ?3 WHERE id = ?1",
        (id, Json(&recipe_ids).to_text()?, event.timestamp as i64),
    )
    .await?;
    Ok(())
}

/// A dish: its name and the recipes it is made of.
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct Dish {
    pub id: String,
    pub name: String,
    /// The recipes it is made of, in the order they are served. One that
    /// was deleted is named until the api takes it out, a moment later.
    pub recipe_ids: Json<Vec<String>>,
    /// Unix seconds.
    pub created_at: i64,
}

impl Dish {
    /// Where the dish stands in its owner's list.
    pub fn cursor(&self) -> Cursor {
        Cursor {
            at: self.created_at,
            id: self.id.clone(),
        }
    }
}

/// The dishes of `owner_id`, the most recent first, `limit` at most: those
/// after `after` in the list, and only those `q` is found in the name of
/// (whatever the case and the accents) when there is one.
pub async fn list_of(
    db: &Db,
    owner_id: &str,
    q: Option<&str>,
    after: Option<&Cursor>,
    limit: i64,
) -> anyhow::Result<Vec<Dish>> {
    let (at, id) = after.map_or((i64::MAX, ""), |cursor| (cursor.at, cursor.id.as_str()));
    db.fetch_all(
        "SELECT id, name, recipe_ids, created_at
         FROM dishes
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

/// The dish `id`, when it is one of `owner_id`'s: someone else's is one
/// that does not exist.
pub async fn get_own(db: &Db, owner_id: &str, id: &str) -> anyhow::Result<Option<Dish>> {
    db.fetch_optional(
        "SELECT id, name, recipe_ids, created_at FROM dishes WHERE id = ?1 AND owner_id = ?2",
        (id, owner_id),
    )
    .await
}
