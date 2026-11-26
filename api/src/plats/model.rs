//! The write side of a plat: what its stream says of it, and the commands
//! of its owner.
//!
//! The state folds every event of the stream (`strict()`), so the version
//! it holds, which a write goes out at, is never stale. It is not
//! snapshotted: a plat is recomposed a few dozen times at most.
//!
//! A plat is made of recipes of its owner, checked when it is composed. A
//! recipe deleted since is taken out of it by a subscription
//! ([`remove_recipes`]), a moment after the fact.

use std::collections::BTreeMap;

use axum::http::StatusCode;
use evento::metadata::Event;
use evento::{Executor, Projection, ProjectionAggregate, WriteBuilder, WriteError};
use imkitchen_aggregates::plat::{Created, Deleted, Edited, Plat, RecipeRemoved};

use crate::error::{ApiError, ApiResult, FieldError};
use crate::recipes::{self, RECIPE_NOT_FOUND};

/// The plat does not exist, was deleted, or is someone else's: all the same
/// to who asks.
pub const PLAT_NOT_FOUND: &str = "plat_not_found";

/// How many recipes a plat is made of at most.
pub const MAX_RECIPES: u64 = 20;

/// The longest id a recipe has; a longer one is no recipe, and not looked
/// for.
const MAX_ID: usize = 64;

/// How many times a write that lost to another on the same plat is made
/// again, over the plat as the other left it.
const WRITE_ATTEMPTS: u32 = 3;

/// What a plat is: its name and the recipes it is made of, in the order
/// they are served.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Composition {
    pub name: String,
    pub recipe_ids: Vec<String>,
}

/// A plat as its stream leaves it.
#[evento::projection(id = id, name = "imkitchen/PlatState")]
#[evento::snapshot(none)]
#[derive(Debug, PartialEq)]
pub struct PlatState {
    pub id: String,
    pub owner_id: String,
    pub composition: Composition,
    /// Unix seconds of `Created`.
    pub created_at: u64,
    /// Unix seconds of the last event.
    pub updated_at: u64,
    pub deleted: bool,
}

fn projection<E: Executor>() -> Projection<E, PlatState> {
    Projection::new::<Plat>()
        .handler(on_created())
        .handler(on_edited())
        .handler(on_deleted())
        .handler(on_recipe_removed())
        .strict()
}

#[evento::handler]
async fn on_created(event: Event<Created>, plat: &mut PlatState) -> anyhow::Result<()> {
    let data = &event.data;
    plat.id = event.aggregate_id.to_owned();
    plat.owner_id = data.owner_id.to_owned();
    plat.composition = Composition {
        name: data.name.to_owned(),
        recipe_ids: data.recipe_ids.to_owned(),
    };
    plat.created_at = event.timestamp;
    plat.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_edited(event: Event<Edited>, plat: &mut PlatState) -> anyhow::Result<()> {
    let data = &event.data;
    plat.composition = Composition {
        name: data.name.to_owned(),
        recipe_ids: data.recipe_ids.to_owned(),
    };
    plat.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_deleted(event: Event<Deleted>, plat: &mut PlatState) -> anyhow::Result<()> {
    plat.deleted = true;
    plat.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_recipe_removed(
    event: Event<RecipeRemoved>,
    plat: &mut PlatState,
) -> anyhow::Result<()> {
    let recipe_ids = &mut plat.composition.recipe_ids;
    recipe_ids.retain(|id| *id != event.data.recipe_id);
    plat.updated_at = event.timestamp;
    Ok(())
}

/// The plat `id` as its stream leaves it, deleted or not.
pub async fn load<E: Executor>(executor: &E, id: &str) -> anyhow::Result<Option<PlatState>> {
    projection().load(id).execute(executor).await
}

/// The plat `id` of `owner_id`. Anyone else's is refused the way one that
/// does not exist is: asking tells nothing of what others composed.
pub async fn load_own<E: Executor>(executor: &E, owner_id: &str, id: &str) -> ApiResult<PlatState> {
    load(executor, id)
        .await?
        .filter(|plat| !plat.deleted && plat.owner_id == owner_id)
        .ok_or(ApiError::refused(StatusCode::NOT_FOUND, PLAT_NOT_FOUND))
}

/// Refuses the recipes that are not `owner_id`'s to compose a plat of, each
/// under its place in the body: one that does not exist, was deleted or is
/// someone else's, all the same to who asks.
async fn check_recipes<E: Executor>(
    executor: &E,
    owner_id: &str,
    recipe_ids: &[String],
) -> ApiResult<()> {
    let mut unknown = Vec::new();
    for (index, id) in recipe_ids.iter().enumerate() {
        if id.len() > MAX_ID || recipes::own(executor, owner_id, id).await?.is_none() {
            unknown.push(FieldError {
                field: format!("recipe_ids[{index}]"),
                code: RECIPE_NOT_FOUND.into(),
                params: BTreeMap::new(),
            });
        }
    }
    if unknown.is_empty() {
        Ok(())
    } else {
        Err(ApiError::Invalid(unknown))
    }
}

/// Composes a plat of `owner_id` under a fresh id, which it answers.
pub async fn create<E: Executor>(
    executor: &E,
    owner_id: &str,
    composition: Composition,
) -> ApiResult<String> {
    check_recipes(executor, owner_id, &composition.recipe_ids).await?;
    let mut write = evento::create();
    write.event(&Created {
        owner_id: owner_id.to_owned(),
        name: composition.name,
        recipe_ids: composition.recipe_ids,
    });
    Ok(write.commit(executor).await?)
}

/// Recomposes the plat `id` of `owner_id` as `composition`; the same as it
/// stands writes nothing.
pub async fn edit<E: Executor>(
    executor: &E,
    owner_id: &str,
    id: &str,
    composition: Composition,
) -> ApiResult<()> {
    // Before its recipes are looked at: someone else's plat says nothing
    // of them.
    load_own(executor, owner_id, id).await?;
    check_recipes(executor, owner_id, &composition.recipe_ids).await?;
    write(executor, owner_id, id, |plat, write| {
        if plat.composition == composition {
            return false;
        }
        let composition = composition.clone();
        write.event(&Edited {
            name: composition.name,
            recipe_ids: composition.recipe_ids,
        });
        true
    })
    .await
}

/// Deletes the plat `id` of `owner_id` for good.
pub async fn delete<E: Executor>(executor: &E, owner_id: &str, id: &str) -> ApiResult<()> {
    write(executor, owner_id, id, |_, write| {
        write.event(&Deleted);
        true
    })
    .await
}

/// Appends what `change` adds to the plat `id` of `owner_id`, at the
/// version it was loaded at; `change` answers `false` when there is nothing
/// to write. A write another one got ahead of is made again over the plat
/// as it now stands.
async fn write<E: Executor>(
    executor: &E,
    owner_id: &str,
    id: &str,
    change: impl Fn(&PlatState, &mut WriteBuilder) -> bool,
) -> ApiResult<()> {
    for _ in 0..WRITE_ATTEMPTS {
        let plat = load_own(executor, owner_id, id).await?;
        let mut write = plat.write()?;
        if !change(&plat, &mut write) {
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

/// Takes the recipes `gone`, which were deleted, out of the plat `id`,
/// whoever's it is. Those it is not made of, or no longer, write nothing,
/// and neither does a plat that was deleted.
pub async fn remove_recipes<E: Executor>(
    executor: &E,
    id: &str,
    gone: &[String],
) -> anyhow::Result<()> {
    for _ in 0..WRITE_ATTEMPTS {
        let Some(plat) = load(executor, id).await?.filter(|plat| !plat.deleted) else {
            return Ok(());
        };
        let mut write = plat.write()?;
        let mut removed = false;
        for recipe_id in gone {
            if plat.composition.recipe_ids.contains(recipe_id) {
                write.event(&RecipeRemoved {
                    recipe_id: recipe_id.to_owned(),
                });
                removed = true;
            }
        }
        if !removed {
            return Ok(());
        }
        match write.commit(executor).await {
            Ok(_) => return Ok(()),
            Err(WriteError::InvalidOriginalVersion) => continue,
            Err(err) => return Err(err.into()),
        }
    }
    anyhow::bail!("the plat {id} kept being written by others")
}
