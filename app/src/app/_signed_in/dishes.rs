//! `/dishes`: the dishes of who reads the page, searched by name, and
//! `/dishes/{dish_id}`: one of them, the recipes it is made of in the order
//! they are served, and the button that deletes it. A dish is composed at
//! `/dishes/new` ([`new`]) and recomposed at `/dishes/{dish_id}/edit`
//! ([`_form`]), by picking recipes.
//!
//! A dish names its recipes: each is shown as it now stands, and one that
//! was deleted is no longer part of it. Someone else's dish is one that
//! does not exist.

pub mod _form;
pub mod new;

use std::collections::HashMap;

use reqwest::Method;
use serde::Deserialize;
use timada_core::Rendition;
use timada_core::page::Cursor;
use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{
        error::{not_found, see_other},
        href, page, path_param, query_params,
    },
    view::{View, class, view},
};

use super::recipes::{self, recipe_id};
use super::{
    BUTTON, BUTTON_QUIET, Failure, H1, H2, LINK, MAIN, account, asking, failed, followed, read,
};
use crate::app::{api, db, language, t};
use crate::dishes::Dish;
use crate::recipes::RecipeRow;

path_param!(pub dish_id);

/// How many dishes a page of the list shows.
const PAGE: usize = 24;

/// What the address of the list says of a dish that was just deleted.
const DELETED: &str = "deleted";

#[query_params]
struct Query {
    q: Option<String>,
    /// Where the page before stopped.
    after: Option<String>,
    /// What was just done to a dish.
    done: Option<String>,
}

/// A dish as the api tells of it (a plat), in answer to the command that
/// composed it or when asked: what app waits to have followed before it
/// shows the dish. No page is read from it.
#[derive(Debug, Deserialize)]
pub(crate) struct Told {
    pub(crate) id: String,
    name: String,
    recipes: Vec<ToldRecipe>,
}

#[derive(Debug, Deserialize)]
struct ToldRecipe {
    id: String,
}

impl Told {
    /// Whether `dish`, as app followed it, says the same.
    fn is(&self, dish: &Dish) -> bool {
        self.name == dish.name
            && self
                .recipes
                .iter()
                .map(|recipe| &recipe.id)
                .eq(&dish.recipe_ids.0)
    }
}

/// Waits until app followed the dish as far as the api tells of it, for
/// the page the browser is sent to to show it as it was just left.
pub(crate) async fn caught_up(cx: &Cx, told: &Told) {
    let (db, owner_id) = (db(cx), account(cx).id.as_str());
    followed(|| async {
        let dish = crate::dishes::get_own(db, owner_id, &told.id).await?;
        Ok(dish.is_some_and(|dish| told.is(&dish)))
    })
    .await;
}

/// Waits until app followed the dish `id` as far as the api has written
/// it. An api that does not say is not waited for: the page shows the dish
/// a moment later.
pub(crate) async fn settled(cx: &Cx, id: &str) {
    let language = language(cx);
    let told = api(cx)
        .read::<Told>(asking(cx, &language), &["plats", id])
        .await;
    if let Ok(told) = told {
        caught_up(cx, &told).await;
    }
}

/// The recipes `ids` name among those of who reads the page, by id: one
/// that was deleted is not among them.
pub(crate) async fn named(cx: &Cx, ids: &[String]) -> Result<HashMap<String, RecipeRow>> {
    let rows = read(crate::recipes::listed_own(db(cx), &account(cx).id, ids).await)?;
    Ok(rows.into_iter().map(|row| (row.id.clone(), row)).collect())
}

/// A dish as the list shows it.
struct Card {
    id: String,
    address: String,
    name: String,
    /// The picture of the first of its recipes that shows one.
    picture: Option<String>,
    /// How many recipes it is made of.
    count: usize,
    /// Their titles, in the order they are served.
    titles: String,
}

#[page(with(Metadata::new().title(t!(dishes_meta_title))))]
pub async fn list(cx: &Cx) -> Result<impl View> {
    let query = query_params::<Query>(cx).ok();
    let q = query
        .and_then(|query| query.q.as_deref())
        .map(str::trim)
        .filter(|q| !q.is_empty());
    let after = query
        .and_then(|query| query.after.as_deref())
        .and_then(Cursor::parse);
    let deleted = query.is_some_and(|query| query.done.as_deref() == Some(DELETED));
    // One more than shown: whether there is a page beyond.
    let mut dishes = read(
        crate::dishes::list_of(db(cx), &account(cx).id, q, after.as_ref(), PAGE as i64 + 1).await,
    )?;
    let more = dishes.len() > PAGE;
    dishes.truncate(PAGE);
    let next = dishes.last().filter(|_| more).map(|last| {
        href!(list)
            .query([("q", q.unwrap_or_default())])
            .query([("after", last.cursor().param())])
            .resolve(cx)
            .to_string()
    });
    // The recipes of the whole page at once.
    let mut recipe_ids: Vec<String> = dishes
        .iter()
        .flat_map(|dish| dish.recipe_ids.0.iter().cloned())
        .collect();
    recipe_ids.sort_unstable();
    recipe_ids.dedup();
    let recipes = named(cx, &recipe_ids).await?;
    let cards: Vec<Card> = dishes
        .into_iter()
        .map(|dish| {
            let parts: Vec<&RecipeRow> = dish
                .recipe_ids
                .0
                .iter()
                .filter_map(|id| recipes.get(id))
                .collect();
            Card {
                address: href!(show, dish_id: dish.id.as_str())
                    .resolve(cx)
                    .to_string(),
                picture: parts
                    .iter()
                    .find_map(|recipe| recipe.file_id.as_deref())
                    .map(|file_id| api(cx).picture(file_id, Rendition::Listing)),
                count: parts.len(),
                titles: parts
                    .iter()
                    .map(|recipe| recipe.title.as_str())
                    .collect::<Vec<_>>()
                    .join(" · "),
                id: dish.id,
                name: dish.name,
            }
        })
        .collect();
    // The first page of everything: the list as it opens.
    let whole = q.is_none() && after.is_none();
    let searched = q.unwrap_or_default().to_owned();

    Ok(view! {
        <main class="mx-auto max-w-5xl px-4 py-8 md:px-8 md:py-12">
            <div class="flex flex-wrap items-center justify-between gap-4">
                <h1 class=(H1)>(t!(dishes_title))</h1>
                <p class="flex gap-3 text-sm">
                    <a class=(BUTTON) href=(href!(new::blank)) data-action="new">
                        (t!(dishes_new))
                    </a>
                </p>
            </div>
            if deleted {
                <p
                    class="mt-6 rounded-lg border border-green-300 bg-green-50 px-4 py-3 text-green-900"
                    role="status"
                    data-done=(DELETED)
                >
                    (t!(dish_deleted))
                </p>
            }
            <form class="mt-6 flex gap-3" method="get" action=(href!(list)) role="search">
                <label class="sr-only" for="dishes-q">(t!(dishes_search_label))</label>
                <input
                    id="dishes-q"
                    class="block w-full rounded-lg border border-stone-300 bg-white px-3 py-2"
                    type="search"
                    name="q"
                    value=(searched)
                    placeholder=(t!(dishes_search_label))
                >
                <button class=(BUTTON_QUIET) type="submit">(t!(recipes_search))</button>
            </form>
            if cards.is_empty() {
                <p class="mt-8 text-stone-600" data-empty="">
                    if whole {
                        (t!(dishes_none))
                    } else {
                        (t!(dishes_none_found))
                    }
                </p>
            }
            <ul class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                for card in cards {
                    <li data-dish=(card.id)>
                        <a
                            class="block h-full overflow-hidden rounded-xl border border-stone-200 bg-white hover:border-stone-400"
                            href=(card.address)
                        >
                            if let Some(picture) = card.picture {
                                <img
                                    class="aspect-4/3 w-full object-cover"
                                    src=(picture)
                                    alt=""
                                    loading="lazy"
                                >
                            } else {
                                <div class="aspect-4/3 w-full bg-stone-100" aria-hidden="true"></div>
                            }
                            <div class="p-4">
                                // A name someone typed: it may be one long
                                // word.
                                <h2 class="font-semibold wrap-break-word">(card.name)</h2>
                                <p class="mt-1 text-sm text-stone-500" data-recipes=(card.count)>
                                    (t!(dish_recipes_count, count: card.count))
                                </p>
                                if card.count > 0 {
                                    <p class="mt-1 line-clamp-2 text-sm text-stone-500">
                                        (card.titles)
                                    </p>
                                }
                            </div>
                        </a>
                    </li>
                }
            </ul>
            if let Some(next) = next {
                <p class="mt-8 text-center">
                    <a class=(BUTTON_QUIET) href=(next) rel="next" data-more="">(t!(dishes_more))</a>
                </p>
            }
        </main>
    })
}

/// A recipe as its dish shows it.
struct Part {
    id: String,
    address: String,
    title: String,
    picture: Option<String>,
    /// From the first thing done to the table: 0 when the recipe says no
    /// time.
    minutes: i64,
    servings: i64,
}

#[page("./{dish_id}", with(Metadata::new().title(t!(dishes_meta_title))))]
pub async fn show(cx: &Cx) -> Result<impl View> {
    let id = dish_id(cx);
    let Some(dish) = read(crate::dishes::get_own(db(cx), &account(cx).id, id).await)? else {
        return Err(not_found().into());
    };
    let mut recipes = named(cx, &dish.recipe_ids.0).await?;
    let parts: Vec<Part> = dish
        .recipe_ids
        .0
        .iter()
        .filter_map(|id| recipes.remove(id))
        .map(|recipe| Part {
            address: href!(recipes::show, recipe_id: recipe.id.as_str())
                .resolve(cx)
                .to_string(),
            picture: recipe
                .file_id
                .as_deref()
                .map(|file_id| api(cx).picture(file_id, Rendition::Listing)),
            id: recipe.id,
            title: recipe.title,
            minutes: recipe.prep_minutes + recipe.cook_minutes,
            servings: recipe.servings,
        })
        .collect();

    Ok(view! {
        <main class=(MAIN) data-dish=(dish.id)>
            <p class="text-sm"><a class=(LINK) href=(href!(list))>(t!(dishes_title))</a></p>
            <h1 class=(class!(H1, "mt-2 wrap-break-word"))>(dish.name)</h1>

            <h2 class=(H2)>(t!(dish_recipes))</h2>
            if parts.is_empty() {
                <p class="mt-4 text-stone-600" data-empty="">(t!(dish_no_recipe))</p>
            }
            <ol class="mt-4 grid gap-3" data-recipes=(parts.len())>
                for part in parts {
                    <li data-recipe=(part.id)>
                        <a
                            class="flex items-center gap-4 overflow-hidden rounded-xl border border-stone-200 bg-white hover:border-stone-400"
                            href=(part.address)
                        >
                            if let Some(picture) = part.picture {
                                <img
                                    class="size-20 shrink-0 object-cover"
                                    src=(picture)
                                    alt=""
                                    loading="lazy"
                                >
                            } else {
                                <div class="size-20 shrink-0 bg-stone-100" aria-hidden="true"></div>
                            }
                            <div class="min-w-0 py-2 pr-4">
                                <h3 class="font-semibold wrap-break-word">(part.title)</h3>
                                <p
                                    class="mt-1 text-sm text-stone-500"
                                    data-minutes=(part.minutes)
                                    data-servings=(part.servings)
                                >
                                    if part.minutes > 0 {
                                        (t!(recipe_minutes, minutes: part.minutes))
                                        " · "
                                    }
                                    (t!(recipe_serves, count: part.servings))
                                </p>
                            </div>
                        </a>
                    </li>
                }
            </ol>

            <div class="mt-12 flex flex-wrap items-start gap-3 border-t border-stone-200 pt-6 text-sm">
                <a class=(BUTTON_QUIET) href=(href!(_form::edit, dish_id: id)) data-action="edit">
                    (t!(recipe_edit))
                </a>
                <details class="rounded-lg border border-stone-300 bg-white px-5 py-3">
                    <summary class="cursor-pointer font-semibold text-red-800">
                        (t!(recipe_delete))
                    </summary>
                    <p class="mt-3 max-w-sm text-stone-700">(t!(dish_delete_warning))</p>
                    <form
                        class="mt-3"
                        method="post"
                        action=(href!(delete, dish_id: id))
                        data-action="delete"
                    >
                        <button
                            class="rounded-lg bg-red-700 px-4 py-2 font-semibold text-white hover:bg-red-800"
                            type="submit"
                        >
                            (t!(dish_delete_confirm))
                        </button>
                    </form>
                </details>
            </div>
        </main>
    })
}

/// Deletes the dish, and sends the browser to the list once app followed
/// that it is gone. Its recipes stay.
#[page(POST "./{dish_id}/delete", with(Metadata::new().title(t!(dishes_meta_title))))]
async fn delete(cx: &Cx) -> Result<impl View> {
    let id = dish_id(cx);
    let (db, owner_id) = (db(cx), account(cx).id.as_str());
    let language = language(cx);
    let deleted = api(cx)
        .command(
            asking(cx, &language),
            Method::DELETE,
            &["plats", id],
            None::<&()>,
        )
        .await;
    let failure = match deleted {
        Ok(()) => {
            followed(|| async { Ok(crate::dishes::get_own(db, owner_id, id).await?.is_none()) })
                .await;
            let then = href!(list).query([("done", DELETED)]).resolve(cx);
            return Err(see_other(then).into());
        }
        Err(err) => Failure::of(cx, err)?,
    };

    Ok(view! {
        <main class=(MAIN)>
            <h1 class=(H1)>(t!(dish_not_deleted))</h1>
            <div class="mt-6">failed(failure: Some(failure))</div>
            <p class="mt-6">
                <a class=(LINK) href=(href!(show, dish_id: id))>(t!(recipe_back))</a>
            </p>
        </main>
    })
}
