//! What app knows of the list of each user: the meals they plan to cook,
//! in the order they were planned.
//!
//! A meal names a recipe or a dish (a plat of the api) and holds nothing
//! of it: a page shows each as app knows it ([`crate::recipes`],
//! [`crate::dishes`]). A list is its owner's alone: it is read by saying
//! whose is asked for.
//!
//! A list is a projection of its stream, under the id of its user
//! ([`ListState`]), whose snapshot is its row of the `lists` table: one row
//! per user whose list was ever filled, kept by a projection subscription
//! ([`subscription`]). A page loads the list of who reads it from the row
//! and the events written after it ([`of`]), so it shows the list as the
//! command its user just handed the api left it.

use evento::Executor;
use evento::metadata::Event;
use evento::projection::{Context, Projection, ProjectionSubscription, Snapshot};
use imkitchen_aggregates::list::{
    self, EntryAdded, EntryRemoved, EntryReplaced, Generated, List, Meal,
};
use serde::{Deserialize, Serialize};
use timada_core::db::{FromRow, Json};

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/list";

/// One meal of a list, as the table keeps it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// Its own id: a recipe or a dish may be on a list more than once.
    pub id: String,
    /// The recipe to cook, when the meal is a recipe.
    #[serde(default)]
    pub recipe_id: Option<String>,
    /// The dish to cook, when the meal is a dish.
    #[serde(default)]
    pub dish_id: Option<String>,
    /// How many people it is cooked for.
    pub servings: u32,
}

impl From<&list::Entry> for Entry {
    fn from(entry: &list::Entry) -> Self {
        let (recipe_id, dish_id) = match &entry.meal {
            Meal::Recipe(id) => (Some(id.clone()), None),
            Meal::Plat(id) => (None, Some(id.clone())),
        };
        Self {
            id: entry.id.clone(),
            recipe_id,
            dish_id,
            servings: entry.servings,
        }
    }
}

/// A list as its stream leaves it.
#[evento::projection]
#[derive(Debug)]
pub struct ListState {
    /// Its meals, in the order they were planned.
    pub entries: Vec<Entry>,
}

fn projection<E: Executor>(db: Db) -> Projection<E, ListState> {
    Projection::new::<List>()
        .handler(on_generated())
        .handler(on_entry_added())
        .handler(on_entry_replaced())
        .handler(on_entry_removed())
        .data(db)
}

/// Keeps the row of each list in `db` as its stream leaves it.
pub fn subscription<E: Executor>(db: Db) -> ProjectionSubscription<E, ListState> {
    projection(db).subscription(SUBSCRIPTION)
}

#[evento::handler]
async fn on_generated(event: Event<Generated>, list: &mut ListState) -> anyhow::Result<()> {
    list.entries = event.data.entries.iter().map(Entry::from).collect();
    Ok(())
}

#[evento::handler]
async fn on_entry_added(event: Event<EntryAdded>, list: &mut ListState) -> anyhow::Result<()> {
    let added = Entry::from(&event.data.entry);
    if !list.entries.iter().any(|entry| entry.id == added.id) {
        list.entries.push(added);
    }
    Ok(())
}

#[evento::handler]
async fn on_entry_replaced(
    event: Event<EntryReplaced>,
    list: &mut ListState,
) -> anyhow::Result<()> {
    let replaced = Entry::from(&event.data.entry);
    if let Some(entry) = list
        .entries
        .iter_mut()
        .find(|entry| entry.id == replaced.id)
    {
        *entry = replaced;
    }
    Ok(())
}

#[evento::handler]
async fn on_entry_removed(event: Event<EntryRemoved>, list: &mut ListState) -> anyhow::Result<()> {
    list.entries.retain(|entry| entry.id != event.data.entry_id);
    Ok(())
}

/// The row of a list, and how far its stream was folded into it.
#[derive(FromRow)]
struct Row {
    entries: Json<Vec<Entry>>,
    cursor: String,
    aggregate_version: i64,
}

impl<E: Executor> Snapshot<E> for ListState {
    async fn restore(context: &Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let db: Db = context.extract();
        let row: Option<Row> = db
            .fetch_optional(
                "SELECT entries, cursor, aggregate_version FROM lists WHERE owner_id = ?1",
                (context.id.as_str(),),
            )
            .await?;
        Ok(row.map(|row| ListState {
            entries: row.entries.0,
            cursor: row.cursor,
            aggregate_version: row.aggregate_version as u16,
            ..Default::default()
        }))
    }

    async fn take_snapshot(&self, context: &Context<'_, E>) -> anyhow::Result<()> {
        let db: Db = context.extract();
        db.execute(
            "INSERT INTO lists (owner_id, entries, cursor, aggregate_version)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (owner_id) DO UPDATE SET entries = excluded.entries,
                cursor = excluded.cursor, aggregate_version = excluded.aggregate_version",
            (
                context.id.as_str(),
                Json(&self.entries).to_text()?,
                self.cursor.as_str(),
                i64::from(self.aggregate_version),
            ),
        )
        .await?;
        Ok(())
    }
}

#[derive(FromRow)]
struct Entries {
    entries: Json<Vec<Entry>>,
}

/// The list of `owner_id`, in the order its meals were planned, as its row
/// holds it: a moment behind the commands. Empty for a user who never
/// filled theirs.
pub async fn row_of(db: &Db, owner_id: &str) -> anyhow::Result<Vec<Entry>> {
    let row: Option<Entries> = db
        .fetch_optional("SELECT entries FROM lists WHERE owner_id = ?1", (owner_id,))
        .await?;
    Ok(row.map(|row| row.entries.0).unwrap_or_default())
}

/// The list of `owner_id`, in the order its meals were planned, as its
/// stream leaves it: its row and the events written after it, read from
/// the api. Empty for a user who never filled theirs.
pub async fn of<E: Executor>(executor: &E, db: &Db, owner_id: &str) -> anyhow::Result<Vec<Entry>> {
    let list = projection(db.clone())
        .load(owner_id)
        .execute(executor)
        .await?;
    Ok(list.map(|list| list.entries).unwrap_or_default())
}
