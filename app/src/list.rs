//! What app knows of the list of each user: the meals they plan to cook,
//! in the order they were planned.
//!
//! A meal names a recipe or a dish (a plat of the api) and holds nothing
//! of it: a page shows each as app knows it ([`crate::recipes`],
//! [`crate::dishes`]). A list is its owner's alone: it is read by saying
//! whose is asked for.
//!
//! app keeps one row per user whose list was ever filled, the meals in it.
//! The handlers leave the row as it should be when an event is delivered
//! twice (the subscription is at-least-once): a generated list is what the
//! event says, a meal is added once however often it is, replaced by what
//! its last event says, and gone however often it is taken off.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::list::{self, EntryAdded, EntryRemoved, EntryReplaced, Generated, Meal};
use serde::{Deserialize, Serialize};
use timada_core::db::{FromRow, Json};

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/list";

/// Follows the lists into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_generated())
        .handler(on_entry_added())
        .handler(on_entry_replaced())
        .handler(on_entry_removed())
}

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

#[derive(FromRow)]
struct Row {
    entries: Json<Vec<Entry>>,
}

/// The list of `owner_id`, in the order its meals were planned, as far as
/// app has followed: empty for a user who never filled theirs.
pub async fn of(db: &Db, owner_id: &str) -> anyhow::Result<Vec<Entry>> {
    let row: Option<Row> = db
        .fetch_optional("SELECT entries FROM lists WHERE owner_id = ?1", (owner_id,))
        .await?;
    Ok(row.map(|row| row.entries.0).unwrap_or_default())
}

async fn write(db: &Db, owner_id: &str, entries: &[Entry]) -> anyhow::Result<()> {
    db.execute(
        "INSERT INTO lists (owner_id, entries) VALUES (?1, ?2)
         ON CONFLICT (owner_id) DO UPDATE SET entries = excluded.entries",
        (owner_id, Json(entries).to_text()?),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_generated<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Generated>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let entries: Vec<Entry> = event.data.entries.iter().map(Entry::from).collect();
    write(&db, &event.aggregate_id, &entries).await
}

#[evento::subscription]
async fn on_entry_added<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<EntryAdded>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let mut entries = of(&db, &event.aggregate_id).await?;
    let added = Entry::from(&event.data.entry);
    if entries.iter().any(|entry| entry.id == added.id) {
        return Ok(());
    }
    entries.push(added);
    write(&db, &event.aggregate_id, &entries).await
}

#[evento::subscription]
async fn on_entry_replaced<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<EntryReplaced>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let mut entries = of(&db, &event.aggregate_id).await?;
    let replaced = Entry::from(&event.data.entry);
    let Some(entry) = entries.iter_mut().find(|entry| entry.id == replaced.id) else {
        return Ok(());
    };
    *entry = replaced;
    write(&db, &event.aggregate_id, &entries).await
}

#[evento::subscription]
async fn on_entry_removed<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<EntryRemoved>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let mut entries = of(&db, &event.aggregate_id).await?;
    entries.retain(|entry| entry.id != event.data.entry_id);
    write(&db, &event.aggregate_id, &entries).await
}
