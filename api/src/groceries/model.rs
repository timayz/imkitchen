//! The write side of a user's groceries: what its stream says of them, and
//! the commands of the user.
//!
//! The stream is under the id of the user, and starts with the first item
//! they check or adjust. It holds what the user says of the items, not the
//! items: those are worked out from the list (`super::items`).
//!
//! What is said of an item is said of the amount the list called for, and
//! kept with it ([`Mark::needed`]): it holds while the list calls for that
//! much, and is nothing once it calls for another. So nothing has to
//! follow the list to keep the groceries right, and a list change that
//! leaves an item as it was leaves what was said of it.
//!
//! The state folds every event of the stream (`strict()`), so the version
//! it holds, which a write goes out at, is never stale. It is not
//! snapshotted.

use std::collections::HashMap;

use axum::http::StatusCode;
use evento::metadata::Event;
use evento::{Executor, Projection, ProjectionAggregate, WriteBuilder, WriteError};
use imkitchen_aggregates::groceries::{Adjusted, Checked, Groceries, Unchecked};

use super::items::{Item, same_amount};
use crate::error::{ApiError, ApiResult};

/// How many times a write that lost to another on the same groceries is
/// made again, over the groceries as the other left them.
const WRITE_ATTEMPTS: u32 = 3;

/// What a user said of one item.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Mark {
    /// How much the list called for when it was said.
    pub needed: Option<f64>,
    /// In the basket.
    pub checked: bool,
    /// How much the user buys, when it is not what the list calls for.
    pub quantity: Option<f64>,
}

/// The groceries of a user as their stream leaves them.
#[evento::projection(id = id, name = "imkitchen/GroceriesState")]
#[evento::snapshot(none)]
#[derive(Debug, PartialEq)]
pub struct GroceriesState {
    /// The id of the user.
    pub id: String,
    /// What was said of each item, by its id. One nothing is said of any
    /// more is not there.
    pub marks: HashMap<String, Mark>,
}

impl GroceriesState {
    /// What the user said of `item` as the list now calls for it: nothing
    /// when it was said of another amount.
    pub fn mark(&self, item: &Item) -> Mark {
        self.marks
            .get(&item.id)
            .filter(|mark| same_amount(mark.needed, item.quantity))
            .copied()
            .unwrap_or_default()
    }

    /// The mark of `item_id` to say something of at `needed`: the one it
    /// has when it was said of that much, a fresh one otherwise.
    fn mark_at(&mut self, item_id: &str, needed: Option<f64>) -> &mut Mark {
        let mark = self.marks.entry(item_id.to_owned()).or_default();
        if !same_amount(mark.needed, needed) {
            *mark = Mark {
                needed,
                ..Mark::default()
            };
        }
        mark
    }

    /// Forgets the mark of `item_id` when it says nothing any more.
    fn settle(&mut self, item_id: &str) {
        let said = |mark: &Mark| mark.checked || mark.quantity.is_some();
        if !self.marks.get(item_id).is_some_and(said) {
            self.marks.remove(item_id);
        }
    }
}

fn projection<E: Executor>() -> Projection<E, GroceriesState> {
    Projection::new::<Groceries>()
        .handler(on_checked())
        .handler(on_unchecked())
        .handler(on_adjusted())
        .strict()
}

#[evento::handler]
async fn on_checked(event: Event<Checked>, groceries: &mut GroceriesState) -> anyhow::Result<()> {
    groceries.id = event.aggregate_id.to_owned();
    groceries
        .mark_at(&event.data.item_id, event.data.needed)
        .checked = true;
    Ok(())
}

#[evento::handler]
async fn on_unchecked(
    event: Event<Unchecked>,
    groceries: &mut GroceriesState,
) -> anyhow::Result<()> {
    if let Some(mark) = groceries.marks.get_mut(&event.data.item_id) {
        mark.checked = false;
    }
    groceries.settle(&event.data.item_id);
    Ok(())
}

#[evento::handler]
async fn on_adjusted(event: Event<Adjusted>, groceries: &mut GroceriesState) -> anyhow::Result<()> {
    groceries.id = event.aggregate_id.to_owned();
    groceries
        .mark_at(&event.data.item_id, event.data.needed)
        .quantity = event.data.quantity;
    groceries.settle(&event.data.item_id);
    Ok(())
}

/// The groceries of `user_id` as their stream leaves them: nothing said of
/// any item for a user who never said anything.
pub async fn load<E: Executor>(executor: &E, user_id: &str) -> anyhow::Result<GroceriesState> {
    let groceries = projection().load(user_id).execute(executor).await?;
    Ok(groceries.unwrap_or_default())
}

/// Puts `item` of the groceries of `user_id` in the basket, or back among
/// what is to buy; as it already is writes nothing.
pub async fn check<E: Executor>(
    executor: &E,
    user_id: &str,
    item: &Item,
    checked: bool,
) -> ApiResult<()> {
    write(executor, user_id, |groceries, write| {
        if groceries.mark(item).checked == checked {
            return false;
        }
        if checked {
            write.event(&Checked {
                item_id: item.id.to_owned(),
                needed: item.quantity,
            });
        } else {
            write.event(&Unchecked {
                item_id: item.id.to_owned(),
            });
        }
        true
    })
    .await
}

/// Has `user_id` buy `quantity` of `item`, `None` for what the list calls
/// for; the same as it stands writes nothing.
pub async fn adjust<E: Executor>(
    executor: &E,
    user_id: &str,
    item: &Item,
    quantity: Option<f64>,
) -> ApiResult<()> {
    write(executor, user_id, |groceries, write| {
        if groceries.mark(item).quantity == quantity {
            return false;
        }
        write.event(&Adjusted {
            item_id: item.id.to_owned(),
            needed: item.quantity,
            quantity,
        });
        true
    })
    .await
}

/// Appends what `change` adds to the groceries of `user_id`, at the
/// version they were loaded at, 0 for a user who never said anything;
/// `change` answers `false` when there is nothing to write. A write
/// another one got ahead of is made again over the groceries as they now
/// stand.
async fn write<E: Executor>(
    executor: &E,
    user_id: &str,
    change: impl Fn(&GroceriesState, &mut WriteBuilder) -> bool,
) -> ApiResult<()> {
    for _ in 0..WRITE_ATTEMPTS {
        let groceries = load(executor, user_id).await?;
        let mut write = evento::append(user_id);
        write.original_version(groceries.aggregate_version()?);
        if !change(&groceries, &mut write) {
            return Ok(());
        }
        match write.commit(executor).await {
            Ok(_) => return Ok(()),
            Err(WriteError::InvalidOriginalVersion) => continue,
            Err(err) => return Err(err.into()),
        }
    }
    Err(ApiError::refused(
        StatusCode::SERVICE_UNAVAILABLE,
        timada_core::WRITE_NOT_TAKEN,
    ))
}
