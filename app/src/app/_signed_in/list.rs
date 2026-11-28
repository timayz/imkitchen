//! `/list`: the meals who reads the page plans to cook, in the order they
//! were planned. The list is generated in one action, drawn by the api as
//! the settings say, or filled by hand: a meal is added at `/list/add` and
//! replaced at `/list/{entry_id}/replace` ([`_pick`]), and taken off from
//! here.
//!
//! A meal is cooked from here, for as many as the list says
//! ([`recipes::_cook`]; a dish asks which of its recipes first).
//!
//! A meal is a recipe or a dish, cooked for so many people. The list names
//! it and keeps nothing of it: each is shown as it now stands, and one
//! that was deleted is no longer on the list.

pub mod _pick;

use std::collections::HashMap;

use imkitchen_aggregates::tutorial::Step;
use reqwest::Method;
use timada_core::Rendition;
use topcoat::{
    Result,
    context::Cx,
    l10n::Translation,
    metadata::Metadata,
    router::{error::see_other, href, page, path_param, query_params},
    view::{View, class, component, view},
};

use super::dishes::{self, dish_id};
use super::recipes::{self, recipe_id};
use super::{BUTTON, BUTTON_QUIET, Failure, H1, account, asking, failed, fresh, read, tutorial};
use crate::api::CommandError;
use crate::app::{api, db, events, language, t};
use crate::list::Entry;

path_param!(pub entry_id);

/// What the address of the list says of what was just done to it.
const GENERATED: &str = "generated";
pub(crate) const ADDED: &str = "added";
pub(crate) const REPLACED: &str = "replaced";
const REMOVED: &str = "removed";

#[query_params]
struct Query {
    /// What was just done to the list.
    done: Option<String>,
}

/// The list of who reads the page, as the commands they handed the api
/// left it.
pub(crate) async fn entries(cx: &Cx) -> Result<Vec<Entry>> {
    let (db, owner_id) = (db(cx), account(cx).id.as_str());
    fresh(
        cx,
        crate::list::of(events(cx), db, owner_id),
        crate::list::row_of(db, owner_id),
    )
    .await
}

/// Sends the browser back to the list, which shows what the api did, once
/// the api did what was asked, saying `done`, and noted `step` of the
/// tutorial when what was asked does one; or says what the api refused.
pub(crate) async fn changed(
    cx: &Cx,
    done: &str,
    step: Option<Step>,
    asked: Result<(), CommandError>,
) -> Result<Failure> {
    match asked {
        Ok(()) => {
            if let Some(step) = step {
                tutorial::noted(cx, step).await;
            }
            let then = href!(show).query([("done", done)]).resolve(cx);
            Err(see_other(then).into())
        }
        Err(err) => Failure::of(cx, err),
    }
}

/// A recipe or a dish, as a meal is shown.
pub(crate) struct Meal {
    pub(crate) title: String,
    /// What a dish is made of: the titles of its recipes, when it has
    /// some.
    pub(crate) made_of: Option<String>,
    pub(crate) picture: Option<String>,
    /// Where it is read.
    pub(crate) address: String,
}

/// The recipes and the dishes of who reads the page among `recipe_ids` and
/// `dish_ids`, as meals, by id: one that was deleted is not among them.
pub(crate) async fn meals(
    cx: &Cx,
    mut recipe_ids: Vec<String>,
    dish_ids: &[String],
) -> Result<(HashMap<String, Meal>, HashMap<String, Meal>)> {
    let found = read(crate::dishes::listed_own(db(cx), &account(cx).id, dish_ids).await)?;
    // The recipes of the dishes with the others, read at once.
    let alone = recipe_ids.clone();
    recipe_ids.extend(
        found
            .iter()
            .flat_map(|dish| dish.recipe_ids.0.iter().cloned()),
    );
    recipe_ids.sort_unstable();
    recipe_ids.dedup();
    let named = dishes::named(cx, &recipe_ids).await?;
    let picture = |file_id: &str| api(cx).picture(file_id, Rendition::Listing);
    let dishes = found
        .into_iter()
        .map(|dish| {
            let parts: Vec<_> = dish
                .recipe_ids
                .0
                .iter()
                .filter_map(|id| named.get(id))
                .collect();
            let meal = Meal {
                made_of: Some(
                    parts
                        .iter()
                        .map(|recipe| recipe.title.as_str())
                        .collect::<Vec<_>>()
                        .join(" · "),
                )
                .filter(|made_of| !made_of.is_empty()),
                picture: parts
                    .iter()
                    .find_map(|recipe| recipe.file_id.as_deref())
                    .map(picture),
                address: href!(dishes::show, dish_id: dish.id.as_str())
                    .resolve(cx)
                    .to_string(),
                title: dish.name,
            };
            (dish.id, meal)
        })
        .collect();
    let recipes = alone
        .iter()
        .filter_map(|id| named.get(id))
        .map(|recipe| {
            let meal = Meal {
                title: recipe.title.clone(),
                made_of: None,
                picture: recipe.file_id.as_deref().map(picture),
                address: href!(recipes::show, recipe_id: recipe.id.as_str())
                    .resolve(cx)
                    .to_string(),
            };
            (recipe.id.clone(), meal)
        })
        .collect();
    Ok((recipes, dishes))
}

/// The meal `entry` names among `recipes` and `dishes`, when it is still
/// there.
pub(crate) fn meal_of<'a>(
    entry: &Entry,
    recipes: &'a HashMap<String, Meal>,
    dishes: &'a HashMap<String, Meal>,
) -> Option<&'a Meal> {
    match (&entry.recipe_id, &entry.dish_id) {
        (Some(id), _) => recipes.get(id),
        (None, Some(id)) => dishes.get(id),
        (None, None) => None,
    }
}

#[page(with(Metadata::new().title(t!(list_meta_title))))]
pub async fn show(cx: &Cx) -> Result<impl View> {
    Ok(view! { content(failure: None) })
}

/// Generates the list in one action: drawn by the api as the settings say,
/// in place of what it held.
#[page(POST "./generate", with(Metadata::new().title(t!(list_meta_title))))]
async fn generate(cx: &Cx) -> Result<impl View> {
    let language = language(cx);
    let generated = api(cx)
        .command(
            asking(cx, &language),
            Method::POST,
            &["list", "generate"],
            None::<&()>,
        )
        .await;
    // The fourth step of the tutorial.
    let failure = changed(cx, GENERATED, Some(Step::List), generated).await?;

    Ok(view! { content(failure: Some(failure)) })
}

/// Takes a meal off the list. Its recipe or its dish stays.
#[page(POST "./{entry_id}/remove", with(Metadata::new().title(t!(list_meta_title))))]
async fn remove(cx: &Cx) -> Result<impl View> {
    let language = language(cx);
    let removed = api(cx)
        .command(
            asking(cx, &language),
            Method::DELETE,
            &["list", "entries", entry_id(cx)],
            None::<&()>,
        )
        .await;
    let failure = changed(cx, REMOVED, None, removed).await?;

    Ok(view! { content(failure: Some(failure)) })
}

/// A meal of the list, as it is shown.
struct Card {
    id: String,
    /// What it is: `recipe` or `dish`.
    kind: &'static str,
    title: String,
    made_of: Option<String>,
    picture: Option<String>,
    address: String,
    servings: u32,
    /// Where it is cooked, for as many as the list says.
    cook: String,
    replace: String,
    remove: String,
}

/// The list, and what `failure` says of the command that was just refused.
#[component]
async fn content(cx: &Cx, failure: Option<Failure>) -> Result<impl View> {
    let entries = entries(cx).await?;
    let ids = |pick: fn(&Entry) -> &Option<String>| -> Vec<String> {
        entries
            .iter()
            .filter_map(|entry| pick(entry).clone())
            .collect()
    };
    let (recipes, dishes) = meals(
        cx,
        ids(|entry| &entry.recipe_id),
        &ids(|entry| &entry.dish_id),
    )
    .await?;
    let cards: Vec<Card> = entries
        .iter()
        .filter_map(|entry| {
            let meal = meal_of(entry, &recipes, &dishes)?;
            let servings = entry.servings.to_string();
            let cook = match (&entry.recipe_id, &entry.dish_id) {
                (Some(id), _) => href!(recipes::_cook::cook, recipe_id: id.as_str())
                    .query([("servings", servings.as_str())])
                    .resolve(cx)
                    .to_string(),
                (None, Some(id)) => href!(dishes::cook, dish_id: id.as_str())
                    .query([("servings", servings.as_str())])
                    .resolve(cx)
                    .to_string(),
                (None, None) => return None,
            };
            Some(Card {
                cook,
                kind: if entry.dish_id.is_some() {
                    "dish"
                } else {
                    "recipe"
                },
                title: meal.title.clone(),
                made_of: meal.made_of.clone(),
                picture: meal.picture.clone(),
                address: meal.address.clone(),
                servings: entry.servings,
                replace: href!(_pick::replace, entry_id: entry.id.as_str())
                    .resolve(cx)
                    .to_string(),
                remove: href!(remove, entry_id: entry.id.as_str())
                    .resolve(cx)
                    .to_string(),
                id: entry.id.clone(),
            })
        })
        .collect();
    let done = query_params::<Query>(cx)
        .ok()
        .and_then(|query| query.done.clone())
        // What a command that was just refused says replaces it.
        .filter(|_| failure.is_none());
    let done: Option<(String, Translation<'static>)> = done.and_then(|done| {
        let said = match done.as_str() {
            GENERATED => t!(list_generated),
            ADDED => t!(list_added),
            REPLACED => t!(list_replaced),
            REMOVED => t!(list_removed),
            _ => return None,
        };
        Some((done, said))
    });
    let count = cards.len();
    let action = "rounded-lg border border-stone-300 bg-white px-3 py-1 text-center text-sm font-semibold hover:border-stone-500";

    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12">
            <div class="flex flex-wrap items-center justify-between gap-4">
                <h1 class=(H1)>(t!(list_title))</h1>
                <div class="flex gap-3 text-sm">
                    <a class=(BUTTON_QUIET) href=(href!(_pick::add)) data-action="add">
                        (t!(list_add))
                    </a>
                    <form method="post" action=(href!(generate)) data-action="generate">
                        <button class=(BUTTON) type="submit">(t!(list_generate))</button>
                    </form>
                </div>
            </div>
            <p class="mt-3 text-sm text-stone-500">(t!(list_generate_hint))</p>
            if let Some((done, said)) = done {
                <p
                    class="mt-6 rounded-lg border border-green-300 bg-green-50 px-4 py-3 text-green-900"
                    role="status"
                    data-done=(done)
                >
                    (said)
                </p>
            }
            <div class="mt-6">failed(failure: failure)</div>
            if count == 0 {
                <p class="mt-8 text-stone-600" data-empty="">(t!(list_none))</p>
            } else {
                <p class="mt-8 text-sm text-stone-500">(t!(list_meals, count: count))</p>
            }
            <ol class="mt-4 grid gap-3" data-entries=(count)>
                for card in cards {
                    <li
                        class="flex items-center gap-4 overflow-hidden rounded-xl border border-stone-200 bg-white"
                        data-entry=(card.id)
                        data-kind=(card.kind)
                    >
                        // As tall as the meal is told, whatever it says.
                        <div class="relative min-h-20 w-20 shrink-0 self-stretch bg-stone-100">
                            if let Some(picture) = card.picture {
                                <img
                                    class="absolute inset-0 size-full object-cover"
                                    src=(picture)
                                    alt=""
                                    loading="lazy"
                                >
                            }
                        </div>
                        <div class="min-w-0 flex-1 py-3">
                            <h2 class="font-semibold wrap-break-word">
                                <a class="hover:underline" href=(card.address)>(card.title.as_str())</a>
                            </h2>
                            if let Some(made_of) = card.made_of {
                                <p class="mt-1 line-clamp-2 text-sm text-stone-500">(made_of)</p>
                            }
                            <p class="mt-1 text-sm text-stone-500" data-servings=(card.servings)>
                                (t!(recipe_serves, count: card.servings))
                            </p>
                        </div>
                        <div class="flex shrink-0 flex-col gap-2 py-3 pr-3 sm:flex-row">
                            <a
                                class="rounded-lg bg-stone-900 px-3 py-1 text-center text-sm font-semibold text-stone-50 hover:bg-stone-700"
                                href=(card.cook)
                                aria-label=(t!(list_cook_of, title: card.title.as_str()))
                                data-action="cook"
                            >
                                (t!(recipe_cook_it))
                            </a>
                            <a
                                class=(action)
                                href=(card.replace)
                                aria-label=(t!(list_replace_of, title: card.title.as_str()))
                            >
                                (t!(list_replace))
                            </a>
                            <form method="post" action=(card.remove)>
                                <button
                                    class=(class!(action, "w-full text-red-800"))
                                    type="submit"
                                    aria-label=(t!(list_remove_of, title: card.title.as_str()))
                                >
                                    (t!(list_remove))
                                </button>
                            </form>
                        </div>
                    </li>
                }
            </ol>
        </main>
    })
}
