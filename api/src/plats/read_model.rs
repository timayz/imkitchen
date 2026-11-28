//! The read side of the plats: the `plats` table a user's list is read
//! from, each row the snapshot of a projection of the plat's stream
//! ([`Listed`]), kept by a projection subscription ([`subscription`]); and
//! the saga that keeps the plats free of the recipes that were deleted
//! ([`saga`]).
//!
//! A deleted plat loses its row. Every node runs both subscriptions over
//! its own table, with cursors of its own.
//!
//! A recipe that is deleted is taken out of the plats it was part of
//! (`RecipeRemoved`, on each of them): those the table knows of when the
//! deletion comes by. A plat composed with the recipe while it was being
//! deleted comes by later; so every event of a plat has its recipes looked
//! at, and those that are gone taken out. The saga does both, in the order
//! of the events: whichever of the two comes last finds the other. For the
//! table to know a plat when the saga reaches the deletion, the saga loads
//! every plat it sees through [`Listed`], which writes its row, rather
//! than counting on the projection subscription to be as far as it is.
//! Taking out what is no longer there writes nothing, so delivery may
//! repeat.

use evento::Executor;
use evento::metadata::{Event, RawEvent};
use evento::projection::{self, Projection, ProjectionSubscription, Snapshot};
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::plat::{Created, Deleted, Edited, Plat, RecipeRemoved};
use imkitchen_aggregates::recipe;
use timada_core::db::{FromRow, Json, search_key};
use timada_core::like_contains;
use timada_core::page::Cursor;

use super::model;
use crate::db::Db;
use crate::recipes;

/// The cursor identity of the subscription that keeps the rows, in the
/// store.
pub const SUBSCRIPTION: &str = "imkitchen-api/plats";

/// The cursor identity of the saga, in the store.
pub const SAGA: &str = "imkitchen-api/plats-recipes";

/// A plat as its row of `plats` holds it: what the lists read of it.
#[evento::projection]
#[derive(Debug)]
pub struct Listed {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    /// The recipes it is made of, in the order they are served.
    pub recipe_ids: Vec<String>,
    /// Unix seconds of `Created`.
    pub created_at: u64,
    /// Unix seconds of the last event.
    pub updated_at: u64,
    pub deleted: bool,
}

fn projection<E: Executor>(db: Db) -> Projection<E, Listed> {
    Projection::new::<Plat>()
        .handler(on_created())
        .handler(on_edited())
        .handler(on_deleted())
        .handler(on_recipe_removed())
        .data(db)
}

/// Keeps the row of each plat in `db` as its stream leaves it. Start it
/// over a handle that reads this node's file as it stands: what it loads is
/// behind the event it handles.
pub fn subscription<E: Executor>(db: Db) -> ProjectionSubscription<E, Listed> {
    projection(db).subscription(SUBSCRIPTION)
}

#[evento::handler]
async fn on_created(event: Event<Created>, plat: &mut Listed) -> anyhow::Result<()> {
    let data = &event.data;
    plat.id = event.aggregate_id.to_owned();
    plat.owner_id = data.owner_id.to_owned();
    plat.name = data.name.to_owned();
    plat.recipe_ids = data.recipe_ids.to_owned();
    plat.created_at = event.timestamp;
    plat.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_edited(event: Event<Edited>, plat: &mut Listed) -> anyhow::Result<()> {
    plat.name = event.data.name.to_owned();
    plat.recipe_ids = event.data.recipe_ids.to_owned();
    plat.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_deleted(event: Event<Deleted>, plat: &mut Listed) -> anyhow::Result<()> {
    plat.deleted = true;
    plat.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_recipe_removed(event: Event<RecipeRemoved>, plat: &mut Listed) -> anyhow::Result<()> {
    plat.recipe_ids.retain(|id| *id != event.data.recipe_id);
    plat.updated_at = event.timestamp;
    Ok(())
}

/// The row of a plat, and how far its stream was folded into it.
#[derive(FromRow)]
struct Row {
    id: String,
    owner_id: String,
    name: String,
    recipe_ids: Json<Vec<String>>,
    created_at: i64,
    updated_at: i64,
    cursor: String,
    aggregate_version: i64,
}

impl<E: Executor> Snapshot<E> for Listed {
    async fn restore(context: &projection::Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let db: Db = context.extract();
        let row: Option<Row> = db
            .fetch_optional(
                "SELECT id, owner_id, name, recipe_ids, created_at, updated_at, cursor,
                    aggregate_version
                 FROM plats WHERE id = ?1",
                (context.id.as_str(),),
            )
            .await?;
        Ok(row.map(|row| Listed {
            id: row.id,
            owner_id: row.owner_id,
            name: row.name,
            recipe_ids: row.recipe_ids.0,
            created_at: row.created_at as u64,
            updated_at: row.updated_at as u64,
            deleted: false,
            cursor: row.cursor,
            aggregate_version: row.aggregate_version as u16,
            ..Default::default()
        }))
    }

    /// A deleted plat loses its row: loading it again folds its stream from
    /// the start, which says it is deleted.
    async fn take_snapshot(&self, context: &projection::Context<'_, E>) -> anyhow::Result<()> {
        let db: Db = context.extract();
        if self.deleted {
            db.execute("DELETE FROM plats WHERE id = ?1", (context.id.as_str(),))
                .await?;
            return Ok(());
        }
        db.execute(
            "INSERT INTO plats (id, owner_id, name, recipe_ids, search, created_at, updated_at,
                cursor, aggregate_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT (id) DO UPDATE SET
                name = excluded.name, recipe_ids = excluded.recipe_ids, search = excluded.search,
                updated_at = excluded.updated_at, cursor = excluded.cursor,
                aggregate_version = excluded.aggregate_version",
            (
                context.id.as_str(),
                self.owner_id.as_str(),
                self.name.as_str(),
                Json(&self.recipe_ids).to_text()?,
                search_key(&self.name),
                self.created_at as i64,
                self.updated_at as i64,
                self.cursor.as_str(),
                i64::from(self.aggregate_version),
            ),
        )
        .await?;
        Ok(())
    }
}

/// Takes the deleted recipes out of the plats, which `db` names by owner.
/// It writes: start it over a handle whose reads wait for the writes before
/// them.
pub fn saga<E: Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SAGA)
        .data(db)
        .handler(on_plat_event())
        .handler(on_recipe_deleted())
        .retry(5)
}

#[evento::subscription_all]
async fn on_plat_event<E: Executor>(
    context: &Context<'_, E>,
    event: RawEvent<Plat>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    // Loaded through the projection, which writes the row: the table knows
    // the plat before the saga handles a later event.
    let plat = projection(db)
        .load(&event.aggregate_id)
        .execute(context.executor)
        .await?;
    let Some(plat) = plat.filter(|plat| !plat.deleted) else {
        return Ok(());
    };

    // Composed with a recipe that was being deleted: its deletion came by
    // before the plat was there to take it out of.
    let mut gone = Vec::new();
    for id in &plat.recipe_ids {
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
async fn on_recipe_deleted<E: Executor>(
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
    for plat in composed_by(&db, &owner_id).await? {
        if plat.recipe_ids.0.contains(recipe_id) {
            model::remove_recipes(context.executor, &plat.id, &gone).await?;
        }
    }
    Ok(())
}

/// What a plat is made of, as the table knows it.
#[derive(FromRow)]
pub struct Composed {
    pub id: String,
    pub recipe_ids: Json<Vec<String>>,
}

/// Every plat of `owner_id` the table knows, in no order.
pub async fn composed_by(db: &Db, owner_id: &str) -> anyhow::Result<Vec<Composed>> {
    db.fetch_all(
        "SELECT id, recipe_ids FROM plats WHERE owner_id = ?1",
        (owner_id,),
    )
    .await
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
