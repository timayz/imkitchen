//! What app knows of the groceries of each user: what there is to buy to
//! cook their list, and what they said of it.
//!
//! What there is to buy is kept nowhere: it is worked out at every read
//! ([`of`]) from the list and its recipes as app knows them
//! ([`crate::list`], [`crate::dishes`], [`crate::recipes`]), the way the api
//! works its own out (`imkitchen_aggregates::groceries::items`), so an item
//! is the same one to both.
//!
//! What a user says of an item is followed from the api's events: that it
//! is in the basket, or that they buy another amount of it. It is said of
//! the amount the list then called for, and holds for as long as it calls
//! for that much.
//!
//! What a user says is a projection of their stream, under their id
//! ([`GroceriesState`]), whose snapshot is their row of the `groceries`
//! table, kept by a projection subscription ([`subscription`]): one row
//! per user who said something, every item they said something of in it.
//! A page loads the groceries of who reads it from their list and their
//! row, and the events written after them ([`of`]).

use std::collections::{BTreeMap, HashMap};

use evento::Executor;
use evento::metadata::Event;
use evento::projection::{Context, Projection, ProjectionSubscription, Snapshot};
use imkitchen_aggregates::groceries::items::{Item, Portion, merge, same_amount};
use imkitchen_aggregates::groceries::{Adjusted, Checked, Groceries, Unchecked};
use imkitchen_aggregates::recipe;
use serde::{Deserialize, Serialize};
use timada_core::db::{FromRow, Json};

use crate::db::Db;
use crate::list::Entry;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/groceries";

/// What a user said of one item, as the table keeps it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
struct Mark {
    /// How much the list called for when it was said.
    needed: Option<f64>,
    /// In the basket.
    checked: bool,
    /// How much the user buys, when it is not what the list calls for.
    quantity: Option<f64>,
}

impl Mark {
    /// The mark to say something of at `needed`: this one when it was said
    /// of that much, a fresh one otherwise.
    fn at(self, needed: Option<f64>) -> Self {
        if same_amount(self.needed, needed) {
            return self;
        }
        Self {
            needed,
            ..Self::default()
        }
    }
}

/// What a user said of their groceries, as their stream leaves it.
#[evento::projection]
#[derive(Debug)]
pub struct GroceriesState {
    /// What they say of each item, by its id: an item that says nothing
    /// any more is forgotten.
    marks: BTreeMap<String, Mark>,
}

impl GroceriesState {
    /// The mark of `item_id`, a fresh one when nothing is said of it.
    fn mark(&self, item_id: &str) -> Option<Mark> {
        self.marks.get(item_id).copied()
    }

    /// Keeps `mark` as what is said of `item_id`.
    fn keep(&mut self, item_id: &str, mark: Mark) {
        if !mark.checked && mark.quantity.is_none() {
            self.marks.remove(item_id);
        } else {
            self.marks.insert(item_id.to_owned(), mark);
        }
    }
}

fn projection<E: Executor>(db: Db) -> Projection<E, GroceriesState> {
    Projection::new::<Groceries>()
        .handler(on_checked())
        .handler(on_unchecked())
        .handler(on_adjusted())
        .data(db)
}

/// Keeps the row of what each user said of their groceries in `db` as their
/// stream leaves it.
pub fn subscription<E: Executor>(db: Db) -> ProjectionSubscription<E, GroceriesState> {
    projection(db).subscription(SUBSCRIPTION)
}

#[evento::handler]
async fn on_checked(event: Event<Checked>, state: &mut GroceriesState) -> anyhow::Result<()> {
    let item_id = event.data.item_id.as_str();
    let mark = state.mark(item_id).unwrap_or_default();
    let mark = Mark {
        checked: true,
        ..mark.at(event.data.needed)
    };
    state.keep(item_id, mark);
    Ok(())
}

#[evento::handler]
async fn on_unchecked(event: Event<Unchecked>, state: &mut GroceriesState) -> anyhow::Result<()> {
    let item_id = event.data.item_id.as_str();
    if let Some(mark) = state.mark(item_id) {
        state.keep(
            item_id,
            Mark {
                checked: false,
                ..mark
            },
        );
    }
    Ok(())
}

#[evento::handler]
async fn on_adjusted(event: Event<Adjusted>, state: &mut GroceriesState) -> anyhow::Result<()> {
    let item_id = event.data.item_id.as_str();
    let mark = state.mark(item_id).unwrap_or_default();
    let mark = Mark {
        quantity: event.data.quantity,
        ..mark.at(event.data.needed)
    };
    state.keep(item_id, mark);
    Ok(())
}

/// The row of what a user said, and how far their stream was folded into
/// it.
#[derive(FromRow)]
struct Row {
    marks: Json<BTreeMap<String, Mark>>,
    cursor: String,
    aggregate_version: i64,
}

impl<E: Executor> Snapshot<E> for GroceriesState {
    async fn restore(context: &Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let db: Db = context.extract();
        let row: Option<Row> = db
            .fetch_optional(
                "SELECT marks, cursor, aggregate_version FROM groceries WHERE owner_id = ?1",
                (context.id.as_str(),),
            )
            .await?;
        Ok(row.map(|row| GroceriesState {
            marks: row.marks.0,
            cursor: row.cursor,
            aggregate_version: row.aggregate_version as u16,
            ..Default::default()
        }))
    }

    async fn take_snapshot(&self, context: &Context<'_, E>) -> anyhow::Result<()> {
        let db: Db = context.extract();
        db.execute(
            "INSERT INTO groceries (owner_id, marks, cursor, aggregate_version)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (owner_id) DO UPDATE SET marks = excluded.marks,
                cursor = excluded.cursor, aggregate_version = excluded.aggregate_version",
            (
                context.id.as_str(),
                Json(&self.marks).to_text()?,
                self.cursor.as_str(),
                i64::from(self.aggregate_version),
            ),
        )
        .await?;
        Ok(())
    }
}

/// One thing to buy, and what its user said of it.
#[derive(Debug, Clone, PartialEq)]
pub struct Grocery {
    /// The same for the same thing in the same unit, whatever the list
    /// holds: what the api knows the item by.
    pub id: String,
    /// As the first recipe that takes it spells it.
    pub name: String,
    pub unit: Option<String>,
    /// How much the list calls for, in `unit`; `None` when no recipe says
    /// ("salt").
    pub quantity: Option<f64>,
    /// How much the user buys instead, when they said so.
    pub adjusted_quantity: Option<f64>,
    /// In the basket.
    pub checked: bool,
}

/// What `owner_id` has to buy to cook the meals of their list, `entries`,
/// as app knows the dishes and the recipes they name.
async fn items(db: &Db, owner_id: &str, entries: &[Entry]) -> anyhow::Result<Vec<Item>> {
    let dish_ids: Vec<String> = entries
        .iter()
        .filter_map(|entry| entry.dish_id.clone())
        .collect();
    let dishes: HashMap<String, Vec<String>> = crate::dishes::listed_own(db, owner_id, &dish_ids)
        .await?
        .into_iter()
        .map(|dish| (dish.id, dish.recipe_ids.0))
        .collect();
    // A dish is each of its recipes; a meal that was deleted and is not off
    // the list yet takes nothing.
    let meals: Vec<(&[String], u32)> = entries
        .iter()
        .map(|entry| {
            let recipe_ids = match (&entry.recipe_id, &entry.dish_id) {
                (Some(id), _) => std::slice::from_ref(id),
                (None, Some(id)) => dishes.get(id).map(Vec::as_slice).unwrap_or_default(),
                (None, None) => &[],
            };
            (recipe_ids, entry.servings)
        })
        .collect();
    // A recipe is read once, however many meals it is part of.
    let mut recipe_ids: Vec<String> = meals
        .iter()
        .flat_map(|(recipe_ids, _)| recipe_ids.iter().cloned())
        .collect();
    recipe_ids.sort_unstable();
    recipe_ids.dedup();
    let recipes: HashMap<String, crate::recipes::Taken> =
        crate::recipes::taken_own(db, owner_id, &recipe_ids)
            .await?
            .into_iter()
            .map(|recipe| (recipe.id.clone(), recipe))
            .collect();
    let mut portions = Vec::new();
    for (recipe_ids, servings) in meals {
        for recipe in recipe_ids.iter().filter_map(|id| recipes.get(id)) {
            let factor = f64::from(servings) / recipe.servings.max(1) as f64;
            portions.extend(recipe.ingredients.0.iter().map(|ingredient| Portion {
                ingredient: recipe::Ingredient {
                    name: ingredient.name.clone(),
                    quantity: ingredient.quantity,
                    unit: ingredient.unit.clone(),
                },
                factor,
            }));
        }
    }
    Ok(merge(portions))
}

/// The groceries of `owner_id`, by name: what the meals of their list,
/// `entries`, take, each item with what they said of it (`marks`) when the
/// list still calls for as much. Empty for an empty list.
async fn worked_out(
    db: &Db,
    owner_id: &str,
    entries: &[Entry],
    marks: &BTreeMap<String, Mark>,
) -> anyhow::Result<Vec<Grocery>> {
    let items = items(db, owner_id, entries).await?;
    Ok(items
        .into_iter()
        .map(|item| {
            let mark = marks
                .get(&item.id)
                .filter(|mark| same_amount(mark.needed, item.quantity));
            Grocery {
                adjusted_quantity: mark.and_then(|mark| mark.quantity),
                checked: mark.is_some_and(|mark| mark.checked),
                id: item.id,
                name: item.name,
                unit: item.unit,
                quantity: item.quantity,
            }
        })
        .collect())
}

#[derive(FromRow)]
struct Said {
    marks: Json<BTreeMap<String, Mark>>,
}

/// The groceries of `owner_id`, by name, from their list and what they said
/// as their rows hold them: a moment behind the commands.
pub async fn row_of(db: &Db, owner_id: &str) -> anyhow::Result<Vec<Grocery>> {
    let entries = crate::list::row_of(db, owner_id).await?;
    let said: Option<Said> = db
        .fetch_optional(
            "SELECT marks FROM groceries WHERE owner_id = ?1",
            (owner_id,),
        )
        .await?;
    let marks = said.map(|said| said.marks.0).unwrap_or_default();
    worked_out(db, owner_id, &entries, &marks).await
}

/// The groceries of `owner_id`, by name, from their list and what they said
/// as their streams leave them: their rows and the events written after
/// them, read from the api.
pub async fn of<E: Executor>(
    executor: &E,
    db: &Db,
    owner_id: &str,
) -> anyhow::Result<Vec<Grocery>> {
    let entries = crate::list::of(executor, db, owner_id).await?;
    let state = projection(db.clone())
        .load(owner_id)
        .execute(executor)
        .await?;
    let marks = state.map(|state| state.marks).unwrap_or_default();
    worked_out(db, owner_id, &entries, &marks).await
}
