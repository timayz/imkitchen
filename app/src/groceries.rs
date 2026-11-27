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
//! app keeps one row per item something is said of. The handlers leave the
//! table as it should be when an event is delivered twice (the subscription
//! is at-least-once): an item is checked however often it is, to buy again
//! however often it is unchecked, and of the amount its last event says.

use std::collections::HashMap;

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::groceries::items::{Item, Portion, merge, same_amount};
use imkitchen_aggregates::groceries::{Adjusted, Checked, Unchecked};
use imkitchen_aggregates::recipe;
use timada_core::db::FromRow;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/groceries";

/// Follows what the users say of their groceries into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_checked())
        .handler(on_unchecked())
        .handler(on_adjusted())
}

/// What a user said of one item, as the table keeps it.
#[derive(Debug, Clone, Copy, Default, PartialEq, FromRow)]
struct Mark {
    /// How much the list called for when it was said.
    needed: Option<f64>,
    /// In the basket: 1, or 0.
    checked: i64,
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

async fn mark_of(db: &Db, owner_id: &str, item_id: &str) -> anyhow::Result<Option<Mark>> {
    db.fetch_optional(
        "SELECT needed, checked, quantity FROM grocery_marks
         WHERE owner_id = ?1 AND item_id = ?2",
        (owner_id, item_id),
    )
    .await
}

/// Keeps `mark` as what `owner_id` says of `item_id`; one that says nothing
/// any more is forgotten.
async fn write(db: &Db, owner_id: &str, item_id: &str, mark: Mark) -> anyhow::Result<()> {
    if mark.checked == 0 && mark.quantity.is_none() {
        db.execute(
            "DELETE FROM grocery_marks WHERE owner_id = ?1 AND item_id = ?2",
            (owner_id, item_id),
        )
        .await?;
        return Ok(());
    }
    db.execute(
        "INSERT INTO grocery_marks (owner_id, item_id, needed, checked, quantity)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (owner_id, item_id) DO UPDATE SET
            needed = excluded.needed, checked = excluded.checked, quantity = excluded.quantity",
        (owner_id, item_id, mark.needed, mark.checked, mark.quantity),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_checked<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Checked>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let (owner_id, item_id) = (event.aggregate_id.as_str(), event.data.item_id.as_str());
    let mark = mark_of(&db, owner_id, item_id).await?.unwrap_or_default();
    let mark = Mark {
        checked: 1,
        ..mark.at(event.data.needed)
    };
    write(&db, owner_id, item_id, mark).await
}

#[evento::subscription]
async fn on_unchecked<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Unchecked>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let (owner_id, item_id) = (event.aggregate_id.as_str(), event.data.item_id.as_str());
    let Some(mark) = mark_of(&db, owner_id, item_id).await? else {
        return Ok(());
    };
    write(&db, owner_id, item_id, Mark { checked: 0, ..mark }).await
}

#[evento::subscription]
async fn on_adjusted<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Adjusted>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let (owner_id, item_id) = (event.aggregate_id.as_str(), event.data.item_id.as_str());
    let mark = mark_of(&db, owner_id, item_id).await?.unwrap_or_default();
    let mark = Mark {
        quantity: event.data.quantity,
        ..mark.at(event.data.needed)
    };
    write(&db, owner_id, item_id, mark).await
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

#[derive(FromRow)]
struct Said {
    item_id: String,
    needed: Option<f64>,
    checked: i64,
    quantity: Option<f64>,
}

/// What `owner_id` has to buy to cook their list, as far as app has
/// followed it.
async fn items(db: &Db, owner_id: &str) -> anyhow::Result<Vec<Item>> {
    let entries = crate::list::of(db, owner_id).await?;
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

/// The groceries of `owner_id`, by name, as far as app has followed: what
/// their list takes as it stands, each item with what they said of it when
/// the list still calls for as much. Empty for an empty list.
pub async fn of(db: &Db, owner_id: &str) -> anyhow::Result<Vec<Grocery>> {
    let items = items(db, owner_id).await?;
    let said: Vec<Said> = db
        .fetch_all(
            "SELECT item_id, needed, checked, quantity FROM grocery_marks WHERE owner_id = ?1",
            (owner_id,),
        )
        .await?;
    let said: HashMap<String, Said> = said
        .into_iter()
        .map(|said| (said.item_id.clone(), said))
        .collect();
    Ok(items
        .into_iter()
        .map(|item| {
            let mark = said
                .get(&item.id)
                .filter(|said| same_amount(said.needed, item.quantity));
            Grocery {
                adjusted_quantity: mark.and_then(|mark| mark.quantity),
                checked: mark.is_some_and(|mark| mark.checked != 0),
                id: item.id,
                name: item.name,
                unit: item.unit,
                quantity: item.quantity,
            }
        })
        .collect())
}
