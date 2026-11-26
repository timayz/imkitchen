//! The write side of a user's list: what its stream says of it, and the
//! commands of the user.
//!
//! The stream is under the id of the user, and starts with the first meal
//! they plan: until then the list is empty.
//!
//! The state folds every event of the stream (`strict()`), so the version
//! it holds, which a write goes out at, is never stale. It is not
//! snapshotted: a list is changed a few times a week.
//!
//! A meal is a recipe or a plat of the user, checked when it is put on the
//! list. One deleted since is taken off by a subscription ([`sweep`]), a
//! moment after the fact.

use std::collections::{BTreeMap, HashSet};

use axum::http::StatusCode;
use evento::metadata::Event;
use evento::{Executor, Projection, ProjectionAggregate, WriteBuilder, WriteError};
use imkitchen_aggregates::list::{
    Entry, EntryAdded, EntryRemoved, EntryReplaced, Generated, List, Meal,
};
use imkitchen_aggregates::settings::ListSources;
use rand::Rng;
use rand::seq::SliceRandom;
use ulid::Ulid;

use crate::db::Db;
use crate::error::{ApiError, ApiResult, FieldError};
use crate::plats::{self, PLAT_NOT_FOUND};
use crate::recipes::{self, RECIPE_NOT_FOUND};
use crate::settings::{self, ListSettings};

/// The list holds no such entry: it never did, or it was taken off.
pub const LIST_ENTRY_NOT_FOUND: &str = "list_entry_not_found";
/// The list holds as many meals as it may.
pub const LIST_FULL: &str = "list_full";
/// The user has nothing a list could be generated from.
pub const NOTHING_TO_GENERATE: &str = "nothing_to_generate";

/// How many meals a list holds at most.
pub const MAX_ENTRIES: usize = 50;

/// The longest id a recipe or a plat has; a longer one is neither, and not
/// looked for.
const MAX_ID: usize = 64;

/// How many times a write that lost to another on the same list is made
/// again, over the list as the other left it.
const WRITE_ATTEMPTS: u32 = 3;

/// The list of a user as its stream leaves it.
#[evento::projection(id = id, name = "imkitchen/ListState")]
#[evento::snapshot(none)]
#[derive(Debug, PartialEq)]
pub struct ListState {
    /// The id of the user.
    pub id: String,
    /// The meals, in the order they were planned.
    pub entries: Vec<Entry>,
}

fn projection<E: Executor>() -> Projection<E, ListState> {
    Projection::new::<List>()
        .handler(on_generated())
        .handler(on_entry_added())
        .handler(on_entry_replaced())
        .handler(on_entry_removed())
        .strict()
}

#[evento::handler]
async fn on_generated(event: Event<Generated>, list: &mut ListState) -> anyhow::Result<()> {
    list.id = event.aggregate_id.to_owned();
    list.entries = event.data.entries.to_owned();
    Ok(())
}

#[evento::handler]
async fn on_entry_added(event: Event<EntryAdded>, list: &mut ListState) -> anyhow::Result<()> {
    list.id = event.aggregate_id.to_owned();
    list.entries.push(event.data.entry.to_owned());
    Ok(())
}

#[evento::handler]
async fn on_entry_replaced(
    event: Event<EntryReplaced>,
    list: &mut ListState,
) -> anyhow::Result<()> {
    let replaced = &event.data.entry;
    for entry in &mut list.entries {
        if entry.id == replaced.id {
            *entry = replaced.to_owned();
        }
    }
    Ok(())
}

#[evento::handler]
async fn on_entry_removed(event: Event<EntryRemoved>, list: &mut ListState) -> anyhow::Result<()> {
    list.entries.retain(|entry| entry.id != event.data.entry_id);
    Ok(())
}

/// The list of `user_id` as its stream leaves it, empty for a user who
/// never filled theirs.
pub async fn load<E: Executor>(executor: &E, user_id: &str) -> anyhow::Result<ListState> {
    let list = projection().load(user_id).execute(executor).await?;
    Ok(list.unwrap_or_default())
}

/// What an entry is asked to be: the meal, and how many it is cooked for,
/// `None` for as many as the user's settings say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    pub meal: Meal,
    pub servings: Option<u32>,
}

/// Whether `meal` is `owner_id`'s to plan: a recipe or a plat of theirs
/// that was not deleted.
async fn is_own<E: Executor>(executor: &E, owner_id: &str, meal: &Meal) -> anyhow::Result<bool> {
    match meal {
        Meal::Recipe(id) if id.len() <= MAX_ID => {
            Ok(recipes::own(executor, owner_id, id).await?.is_some())
        }
        Meal::Plat(id) if id.len() <= MAX_ID => plats::is_own(executor, owner_id, id).await,
        _ => Ok(false),
    }
}

/// `pick` as the entry `id` of the list of `owner_id`. Refused under its
/// field when the meal is not theirs to plan: one that does not exist, was
/// deleted or is someone else's, all the same to who asks.
async fn entry<E: Executor>(
    executor: &E,
    owner_id: &str,
    id: &str,
    pick: Pick,
) -> ApiResult<Entry> {
    if !is_own(executor, owner_id, &pick.meal).await? {
        let (field, code) = match pick.meal {
            Meal::Recipe(_) => ("recipe_id", RECIPE_NOT_FOUND),
            Meal::Plat(_) => ("plat_id", PLAT_NOT_FOUND),
        };
        return Err(ApiError::Invalid(vec![FieldError {
            field: field.to_owned(),
            code: code.into(),
            params: BTreeMap::new(),
        }]));
    }
    let servings = match pick.servings {
        Some(servings) => servings,
        None => settings::of(executor, owner_id).await?.servings,
    };
    Ok(Entry {
        id: id.to_owned(),
        meal: pick.meal,
        servings,
    })
}

/// Puts `pick` on the list of `owner_id`, after what it holds, and answers
/// the entry it is.
pub async fn add<E: Executor>(executor: &E, owner_id: &str, pick: Pick) -> ApiResult<Entry> {
    let entry = entry(executor, owner_id, &Ulid::generate().to_string(), pick).await?;
    write(executor, owner_id, |list, write| {
        if list.entries.len() >= MAX_ENTRIES {
            return Err(ApiError::refused(
                StatusCode::UNPROCESSABLE_ENTITY,
                LIST_FULL,
            ));
        }
        write.event(&EntryAdded {
            entry: entry.clone(),
        });
        Ok(true)
    })
    .await?;
    Ok(entry)
}

fn entry_not_found() -> ApiError {
    ApiError::refused(StatusCode::NOT_FOUND, LIST_ENTRY_NOT_FOUND)
}

/// Makes the entry `entry_id` of the list of `owner_id` what `pick` says,
/// in the place it has; the same as it stands writes nothing.
pub async fn replace<E: Executor>(
    executor: &E,
    owner_id: &str,
    entry_id: &str,
    pick: Pick,
) -> ApiResult<()> {
    // Before the meal is looked at: an entry that is not there says
    // nothing of it.
    let list = load(executor, owner_id).await?;
    if !list.entries.iter().any(|entry| entry.id == entry_id) {
        return Err(entry_not_found());
    }
    let entry = entry(executor, owner_id, entry_id, pick).await?;
    write(executor, owner_id, |list, write| {
        let held = list.entries.iter().find(|held| held.id == entry.id);
        let held = held.ok_or_else(entry_not_found)?;
        if *held == entry {
            return Ok(false);
        }
        write.event(&EntryReplaced {
            entry: entry.clone(),
        });
        Ok(true)
    })
    .await
}

/// Takes the entry `entry_id` off the list of `owner_id`.
pub async fn remove<E: Executor>(executor: &E, owner_id: &str, entry_id: &str) -> ApiResult<()> {
    write(executor, owner_id, |list, write| {
        if !list.entries.iter().any(|entry| entry.id == entry_id) {
            return Err(entry_not_found());
        }
        write.event(&EntryRemoved {
            entry_id: entry_id.to_owned(),
        });
        Ok(true)
    })
    .await
}

/// What a list may be drawn from, going by `sources`: the plats that are
/// made of something, and the recipes. With both, a recipe that is part of
/// a plat is not a meal of its own: it comes with its plat.
fn pool(sources: ListSources, plats: &[(String, Vec<String>)], recipes: &[String]) -> Vec<Meal> {
    let composed = plats
        .iter()
        .filter(|(_, recipe_ids)| !recipe_ids.is_empty());
    let plat_meals = composed.clone().map(|(id, _)| Meal::Plat(id.to_owned()));
    let recipe_meals = recipes.iter().map(|id| Meal::Recipe(id.to_owned()));
    match sources {
        ListSources::Plats => plat_meals.collect(),
        ListSources::Recipes => recipe_meals.collect(),
        ListSources::PlatsAndRecipes => {
            let parts: HashSet<&String> = composed.flat_map(|(_, recipe_ids)| recipe_ids).collect();
            let alone = recipes.iter().filter(|id| !parts.contains(id));
            plat_meals
                .chain(alone.map(|id| Meal::Recipe(id.to_owned())))
                .collect()
        }
    }
}

/// `meals` of `pool` at most, none twice, at random: those the list does
/// not hold as it stands (`held`) before those it does, so that a list
/// generated again changes as much as it can.
fn draw(mut pool: Vec<Meal>, meals: usize, held: &[Entry], rng: &mut impl Rng) -> Vec<Meal> {
    pool.shuffle(rng);
    pool.sort_by_key(|meal| held.iter().any(|entry| entry.meal == *meal));
    pool.truncate(meals);
    pool
}

/// Generates the list of `owner_id` as their settings say, from the
/// recipes and the plats `db` lists of them, in place of what it held.
/// It holds fewer meals than asked when there are not as many to draw.
pub async fn generate<E: Executor>(executor: &E, db: &Db, owner_id: &str) -> ApiResult<()> {
    let ListSettings {
        servings,
        meals,
        sources,
    } = settings::of(executor, owner_id).await?;
    let plats = plats::composed_by(db, owner_id).await?;
    let recipes = recipes::ids_of(db, owner_id).await?;
    let pool = pool(sources, &plats, &recipes);
    if pool.is_empty() {
        return Err(ApiError::refused(
            StatusCode::UNPROCESSABLE_ENTITY,
            NOTHING_TO_GENERATE,
        ));
    }
    write(executor, owner_id, |list, write| {
        let drawn = draw(
            pool.clone(),
            meals as usize,
            &list.entries,
            &mut rand::rng(),
        );
        let entries = drawn
            .into_iter()
            .map(|meal| Entry {
                id: Ulid::generate().to_string(),
                meal,
                servings,
            })
            .collect();
        write.event(&Generated { entries });
        Ok(true)
    })
    .await
}

/// Appends what `change` adds to the list of `user_id`, at the version it
/// was loaded at, 0 for a list never filled; `change` answers `false` when
/// there is nothing to write. A write another one got ahead of is made
/// again over the list as it now stands.
async fn write<E: Executor>(
    executor: &E,
    user_id: &str,
    change: impl Fn(&ListState, &mut WriteBuilder) -> ApiResult<bool>,
) -> ApiResult<()> {
    for _ in 0..WRITE_ATTEMPTS {
        let list = load(executor, user_id).await?;
        let mut write = evento::append(user_id);
        write.original_version(list.aggregate_version()?);
        if !change(&list, &mut write)? {
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

/// Takes off the list of `user_id` the meals that are no longer theirs to
/// plan: those whose recipe or plat was deleted. A list that holds none
/// writes nothing.
pub async fn sweep<E: Executor>(executor: &E, user_id: &str) -> anyhow::Result<()> {
    for _ in 0..WRITE_ATTEMPTS {
        let list = load(executor, user_id).await?;
        let mut write = evento::append(user_id);
        write.original_version(list.aggregate_version()?);
        let mut removed = false;
        for entry in &list.entries {
            if !is_own(executor, user_id, &entry.meal).await? {
                write.event(&EntryRemoved {
                    entry_id: entry.id.to_owned(),
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
    anyhow::bail!("the list of {user_id} kept being written by others")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    fn recipe(id: &str) -> Meal {
        Meal::Recipe(id.to_owned())
    }

    fn plat(id: &str) -> Meal {
        Meal::Plat(id.to_owned())
    }

    #[test]
    fn the_pool_is_what_the_sources_say() {
        let plats = [
            ("roast".to_owned(), ids(&["chicken", "gravy"])),
            ("emptied".to_owned(), Vec::new()),
        ];
        let recipes = ids(&["chicken", "gravy", "soup"]);

        assert_eq!(
            pool(ListSources::Plats, &plats, &recipes),
            [plat("roast")],
            "a plat made of nothing is no meal"
        );
        assert_eq!(
            pool(ListSources::Recipes, &plats, &recipes),
            [recipe("chicken"), recipe("gravy"), recipe("soup")]
        );
        assert_eq!(
            pool(ListSources::PlatsAndRecipes, &plats, &recipes),
            [plat("roast"), recipe("soup")],
            "a recipe of a plat comes with its plat"
        );
        assert_eq!(
            pool(ListSources::PlatsAndRecipes, &[], &recipes).len(),
            3,
            "without plats, every recipe"
        );
    }

    #[test]
    fn a_draw_holds_no_meal_twice_and_what_is_new_first() {
        let all: Vec<Meal> = (0..10).map(|n| recipe(&n.to_string())).collect();
        let held: Vec<Entry> = all[..6]
            .iter()
            .map(|meal| Entry {
                id: String::new(),
                meal: meal.clone(),
                servings: 4,
            })
            .collect();
        let mut rng = rand::rng();

        for _ in 0..20 {
            let drawn = draw(all.clone(), 7, &held, &mut rng);
            assert_eq!(drawn.len(), 7);
            let distinct: HashSet<&Meal> = drawn.iter().collect();
            assert_eq!(distinct.len(), 7);
            assert!(
                all[6..].iter().all(|new| drawn.contains(new)),
                "the four the list did not hold are all drawn"
            );
        }
        assert_eq!(
            draw(all.clone(), 31, &[], &mut rng).len(),
            10,
            "all there is"
        );
    }
}
