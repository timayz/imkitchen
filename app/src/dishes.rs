//! What app knows of the dishes (the plats of the api): the name of each
//! one, whose it is and the recipes it is made of, in the order they are
//! served.
//!
//! A dish names its recipes and holds nothing of them: a page shows each
//! as the recipes of app know it ([`crate::recipes`]). Like them a dish is
//! its owner's alone: nothing is read without saying whose is asked for.
//!
//! A dish is a projection of its stream ([`DishState`]) whose snapshot is
//! its row of the `dishes` table: one row per dish that is not deleted,
//! kept by a projection subscription ([`subscription`]). One dish is loaded
//! from its row and the events written after it ([`get_own`]), so a page
//! shows it as the command its user just handed the api left it. The lists
//! are read from the table, a moment behind the commands.

use evento::Executor;
use evento::metadata::Event;
use evento::projection::{Context, Projection, ProjectionSubscription, Snapshot};
use imkitchen_aggregates::plat::{Created, Deleted, Edited, Plat, RecipeRemoved};
use timada_core::db::{FromRow, Json, params_from_iter, placeholders, search_key};
use timada_core::like_contains;
use timada_core::page::Cursor;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/dishes";

/// A dish as its stream leaves it.
#[evento::projection]
#[derive(Debug)]
pub struct DishState {
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

fn projection<E: Executor>(db: Db) -> Projection<E, DishState> {
    Projection::new::<Plat>()
        .handler(on_created())
        .handler(on_edited())
        .handler(on_deleted())
        .handler(on_recipe_removed())
        .data(db)
}

/// Keeps the row of each dish in `db` as its stream leaves it.
pub fn subscription<E: Executor>(db: Db) -> ProjectionSubscription<E, DishState> {
    projection(db).subscription(SUBSCRIPTION)
}

#[evento::handler]
async fn on_created(event: Event<Created>, dish: &mut DishState) -> anyhow::Result<()> {
    let data = &event.data;
    dish.id = event.aggregate_id.to_owned();
    dish.owner_id = data.owner_id.to_owned();
    dish.name = data.name.to_owned();
    dish.recipe_ids = data.recipe_ids.to_owned();
    dish.created_at = event.timestamp;
    dish.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_edited(event: Event<Edited>, dish: &mut DishState) -> anyhow::Result<()> {
    dish.name = event.data.name.to_owned();
    dish.recipe_ids = event.data.recipe_ids.to_owned();
    dish.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_deleted(event: Event<Deleted>, dish: &mut DishState) -> anyhow::Result<()> {
    dish.deleted = true;
    dish.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_recipe_removed(
    event: Event<RecipeRemoved>,
    dish: &mut DishState,
) -> anyhow::Result<()> {
    dish.recipe_ids.retain(|id| *id != event.data.recipe_id);
    dish.updated_at = event.timestamp;
    Ok(())
}

/// The row of a dish, and how far its stream was folded into it.
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

impl<E: Executor> Snapshot<E> for DishState {
    async fn restore(context: &Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let db: Db = context.extract();
        let row: Option<Row> = db
            .fetch_optional(
                "SELECT id, owner_id, name, recipe_ids, created_at, updated_at, cursor,
                    aggregate_version
                 FROM dishes WHERE id = ?1",
                (context.id.as_str(),),
            )
            .await?;
        Ok(row.map(|row| DishState {
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

    /// A deleted dish loses its row: loading it again folds its stream from
    /// the start, which says it is deleted.
    async fn take_snapshot(&self, context: &Context<'_, E>) -> anyhow::Result<()> {
        let db: Db = context.extract();
        if self.deleted {
            db.execute("DELETE FROM dishes WHERE id = ?1", (context.id.as_str(),))
                .await?;
            return Ok(());
        }
        db.execute(
            "INSERT INTO dishes (id, owner_id, name, recipe_ids, search, created_at, updated_at,
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

/// The dish `id`, when it is one of `owner_id`'s, as its row holds it: a
/// moment behind the commands. Someone else's is one that does not exist.
pub async fn row_own(db: &Db, owner_id: &str, id: &str) -> anyhow::Result<Option<Dish>> {
    db.fetch_optional(
        "SELECT id, name, recipe_ids, created_at FROM dishes WHERE id = ?1 AND owner_id = ?2",
        (id, owner_id),
    )
    .await
}

/// The dish `id`, when it is one of `owner_id`'s, as its stream leaves it:
/// its row and the events written after it, read from the api. Someone
/// else's is one that does not exist.
pub async fn get_own<E: Executor>(
    executor: &E,
    db: &Db,
    owner_id: &str,
    id: &str,
) -> anyhow::Result<Option<Dish>> {
    let dish = projection(db.clone()).load(id).execute(executor).await?;
    Ok(dish
        .filter(|dish| !dish.deleted && dish.owner_id == owner_id)
        .map(|dish| Dish {
            id: dish.id,
            name: dish.name,
            recipe_ids: Json(dish.recipe_ids),
            created_at: dish.created_at as i64,
        }))
}

/// The dishes `ids` name, in no order: those among them that are
/// `owner_id`'s.
pub async fn listed_own(db: &Db, owner_id: &str, ids: &[String]) -> anyhow::Result<Vec<Dish>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut params: Vec<&str> = ids.iter().map(String::as_str).collect();
    params.push(owner_id);
    db.fetch_all(
        &format!(
            "SELECT id, name, recipe_ids, created_at
             FROM dishes WHERE id IN ({}) AND owner_id = ?{}",
            placeholders(ids.len()),
            params.len()
        ),
        params_from_iter(params),
    )
    .await
}
