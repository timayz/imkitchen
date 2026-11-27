//! The form a dish is composed with, at `/dishes/new` ([`super::new`]) and
//! at `/dishes/{dish_id}/edit`, where one is recomposed.
//!
//! A dish is a name and recipes of who composes it, picked among theirs:
//! the form shows those of the dish, in the order they are served, then
//! the others, the most recent first or those a search finds. A recipe is
//! part of the dish while its box is ticked.
//!
//! The form works without a script. Searching, and moving a recipe up or
//! down, post the form, which comes back as it was left: the recipes that
//! were ticked among the others have joined those of the dish, after them,
//! and those that were unticked have left it. Nothing is written until the
//! form is saved.
//!
//! What was picked goes to the api as it was picked: app decides nothing of
//! what a dish may hold. What the api refuses is said under the name, or
//! under the recipes.
//!
//! A dish composed is the second step of the tutorial ([`tutorial`]).

use imkitchen_aggregates::tutorial::Step;
use reqwest::Method;
use serde::Serialize;
use timada_core::Rendition;
use topcoat::{
    Result,
    context::Cx,
    l10n::Translation,
    metadata::Metadata,
    router::{
        content::Form,
        error::{not_found, see_other},
        href, page,
    },
    view::{View, component, view},
};

use super::{Told, caught_up, dish_id, list, named, settled, show};
use crate::app::_signed_in::{
    BUTTON, BUTTON_QUIET, FIELD_REFUSED, Failure, H1, H2, INPUT, LABEL, LINK, MAIN, account,
    asking, failed, read, recipes, tutorial,
};
use crate::app::{api, db, language, t};
use crate::dishes::Dish;

/// How many of the recipes that are not part of the dish the form shows:
/// the others are searched for.
const OTHERS: usize = 24;

/// What a post asks for when it is not saving the dish.
#[derive(Debug, PartialEq)]
enum Asked {
    /// The other recipes, searched.
    Search,
    /// The recipe to be served before the one it follows.
    Up(String),
    /// The recipe to be served after the one that follows it.
    Down(String),
}

/// Everything the form holds, as it was left.
#[derive(Default)]
pub(crate) struct Typed {
    name: String,
    /// The recipes whose box is ticked: those of the dish in the order they
    /// are served, then those that join it.
    recipe_ids: Vec<String>,
    /// What the other recipes are searched for.
    q: String,
    asked: Option<Asked>,
}

/// What composing a dish and recomposing one both take: the body of
/// `POST /v1/plats` and of `PUT /v1/plats/{id}`.
#[derive(Serialize)]
struct DishBody<'a> {
    name: &'a str,
    recipe_ids: &'a [String],
}

impl Typed {
    /// The form of `dish`, as it stands.
    fn of(dish: Dish) -> Self {
        Self {
            name: dish.name,
            recipe_ids: dish.recipe_ids.0,
            ..Self::default()
        }
    }

    /// What the form posted, a recipe moved when one was asked to be.
    fn posted(fields: Vec<(String, String)>) -> Self {
        let mut typed = Self::default();
        for (name, value) in fields {
            match name.as_str() {
                "name" => typed.name = value,
                "recipe_ids" => typed.recipe_ids.push(value),
                "q" => typed.q = value,
                "search" => typed.asked = Some(Asked::Search),
                "up" => typed.asked = Some(Asked::Up(value)),
                "down" => typed.asked = Some(Asked::Down(value)),
                _ => {}
            }
        }
        let ids = &mut typed.recipe_ids;
        let at = |moved: &String| ids.iter().position(|id| id == moved);
        match &typed.asked {
            Some(Asked::Up(moved)) => {
                if let Some(at) = at(moved).filter(|at| *at > 0) {
                    ids.swap(at, at - 1);
                }
            }
            Some(Asked::Down(moved)) => {
                if let Some(at) = at(moved).filter(|at| at + 1 < ids.len()) {
                    ids.swap(at, at + 1);
                }
            }
            Some(Asked::Search) | None => {}
        }
        typed
    }

    fn body(&self) -> DishBody<'_> {
        DishBody {
            name: &self.name,
            recipe_ids: &self.recipe_ids,
        }
    }
}

/// How a form that was posted went, when it did not send the browser on.
pub(crate) struct Again {
    typed: Typed,
    failure: Option<Failure>,
}

impl Again {
    /// A form nothing was posted of yet.
    pub(crate) fn of(typed: Typed) -> Self {
        Self {
            typed,
            failure: None,
        }
    }
}

/// Hands what the form posted to the api: a dish to compose without `id`,
/// the dish `id` to recompose with one. Once the api did it and app
/// followed, the step of the tutorial with it when the dish is a new one,
/// the browser is sent to the dish; anything else is the form again.
pub(crate) async fn hand(
    cx: &Cx,
    id: Option<&str>,
    posted: Vec<(String, String)>,
) -> Result<Again> {
    let typed = Typed::posted(posted);
    if typed.asked.is_some() {
        return Ok(Again::of(typed));
    }
    let language = language(cx);
    let asking = asking(cx, &language);
    let written = match id {
        Some(id) => {
            let recomposed = api(cx)
                .command(asking, Method::PUT, &["plats", id], Some(&typed.body()))
                .await;
            if recomposed.is_ok() {
                settled(cx, id).await;
            }
            recomposed.map(|()| id.to_owned())
        }
        None => {
            let composed = api(cx)
                .create::<_, Told>(asking, &["plats"], &typed.body())
                .await;
            if let Ok(told) = &composed {
                tokio::join!(caught_up(cx, told), tutorial::noted(cx, Step::Plat));
            }
            composed.map(|told| told.id)
        }
    };
    match written {
        Ok(id) => Err(see_other(href!(show, dish_id: id.as_str()).resolve(cx)).into()),
        Err(err) => Ok(Again {
            failure: Some(Failure::of(cx, err)?),
            typed,
        }),
    }
}

#[page("./{dish_id}/edit", with(Metadata::new().title(t!(dish_edit_meta_title))))]
pub async fn edit(cx: &Cx) -> Result<impl View> {
    let id = dish_id(cx);
    let Some(dish) = read(crate::dishes::get_own(db(cx), &account(cx).id, id).await)? else {
        return Err(not_found().into());
    };

    Ok(view! {
        form(again: Again::of(Typed::of(dish)), recomposes: Some(id.to_owned()))
    })
}

#[page(POST "./{dish_id}/edit", with(Metadata::new().title(t!(dish_edit_meta_title))))]
async fn submit(cx: &Cx, Form(posted): Form<Vec<(String, String)>>) -> Result<impl View> {
    let id = dish_id(cx);
    let again = hand(cx, Some(id), posted).await?;

    Ok(view! { form(again: again, recomposes: Some(id.to_owned())) })
}

/// A recipe as the form shows it.
struct Row {
    id: String,
    title: String,
    picture: Option<String>,
    /// What the api refused of it.
    refused: Option<String>,
    /// Whether it is the first, or the last, of those of the dish.
    first: bool,
    last: bool,
}

/// The form, as it was left when it comes back: the one that recomposes
/// the dish `recomposes`, or the one that composes a dish without.
#[component]
pub(crate) async fn form(cx: &Cx, again: Again, recomposes: Option<String>) -> Result<impl View> {
    let Again { typed, failure } = again;
    let (action, back, title): (String, String, Translation<'static>) = match recomposes.as_deref()
    {
        Some(id) => (
            href!(submit, dish_id: id).resolve(cx).to_string(),
            href!(show, dish_id: id).resolve(cx).to_string(),
            t!(dish_edit_title),
        ),
        None => (
            href!(super::new::submit).resolve(cx).to_string(),
            href!(list).resolve(cx).to_string(),
            t!(dish_new_title),
        ),
    };
    let refused = |field: &str| failure.as_ref().and_then(|failure| failure.field(field));
    let name_refused = refused("name");
    let picture =
        |file_id: Option<&str>| file_id.map(|file_id| api(cx).picture(file_id, Rendition::Listing));

    // The recipes of the dish, as they now stand. One that is no longer
    // there (it was deleted) has no row: what the api says of it is said
    // of the recipes as a whole.
    let mut known = named(cx, &typed.recipe_ids).await?;
    let mut recipes_refused: Vec<String> = refused("recipe_ids").into_iter().collect();
    let mut chosen = Vec::new();
    for (at, id) in typed.recipe_ids.iter().enumerate() {
        let said = refused(&format!("recipe_ids[{at}]"));
        match known.remove(id) {
            Some(recipe) => chosen.push(Row {
                picture: picture(recipe.file_id.as_deref()),
                id: recipe.id,
                title: recipe.title,
                refused: said,
                first: false,
                last: false,
            }),
            None => recipes_refused.extend(said.filter(|said| !recipes_refused.contains(said))),
        }
    }
    if let Some(row) = chosen.first_mut() {
        row.first = true;
    }
    if let Some(row) = chosen.last_mut() {
        row.last = true;
    }

    // The others: the most recent, or those the search finds. Those of the
    // dish are read with them, to be left out.
    let q = Some(typed.q.trim()).filter(|q| !q.is_empty());
    let limit = OTHERS + typed.recipe_ids.len() + 1;
    let found =
        read(crate::recipes::list_of(db(cx), &account(cx).id, q, None, limit as i64).await)?;
    let mut others: Vec<Row> = found
        .into_iter()
        .filter(|recipe| !typed.recipe_ids.contains(&recipe.id))
        .map(|recipe| Row {
            picture: picture(recipe.file_id.as_deref()),
            id: recipe.id,
            title: recipe.title,
            refused: None,
            first: false,
            last: false,
        })
        .collect();
    let more = others.len() > OTHERS;
    others.truncate(OTHERS);
    // Nothing to pick, and nothing was searched for: no recipe at all.
    let no_recipe = chosen.is_empty() && others.is_empty() && q.is_none();
    let none_chosen = chosen.is_empty();
    let none_found = others.is_empty();
    let move_button = "rounded-lg border border-stone-300 bg-white px-3 py-1 font-semibold hover:border-stone-500 disabled:opacity-30";
    let row = "flex items-center gap-3 rounded-lg border border-stone-200 bg-white px-3 py-2";
    let thumbnail = "size-10 shrink-0 rounded object-cover";

    Ok(view! {
        <main class=(MAIN)>
            <p class="text-sm"><a class=(LINK) href=(back.as_str())>(t!(recipe_back))</a></p>
            <h1 class=(H1)>(title)</h1>
            <form class="mt-8 grid gap-5" method="post" action=(action) data-dish=(recomposes)>
                // What the Enter key does in a field: a search, which writes
                // nothing. A browser takes the first button of a form.
                <button
                    class="sr-only"
                    type="submit"
                    name="search"
                    value="1"
                    tabindex="-1"
                    aria-hidden="true"
                    formnovalidate=""
                ></button>
                failed(failure: failure)
                <div>
                    <label class=(LABEL) for="dish-name">(t!(dish_name))</label>
                    <input
                        id="dish-name"
                        class=(INPUT)
                        type="text"
                        name="name"
                        value=(typed.name)
                        aria-invalid=(name_refused.is_some().then_some("true"))
                        required=""
                    >
                    if let Some(message) = name_refused {
                        <p class=(FIELD_REFUSED) data-field="name">(message)</p>
                    }
                </div>

                <fieldset>
                    <legend class=(H2)>(t!(dish_chosen))</legend>
                    <p class="mt-1 text-sm text-stone-500">(t!(dish_chosen_hint))</p>
                    for message in recipes_refused {
                        <p class=(FIELD_REFUSED) data-field="recipe_ids">(message)</p>
                    }
                    if none_chosen {
                        <p class="mt-4 text-stone-600" data-chosen="none">(t!(dish_chosen_none))</p>
                    }
                    <ol class="mt-4 grid gap-2" data-rows="chosen">
                        for (at, recipe) in chosen.into_iter().enumerate() {
                            <li data-chosen=(recipe.id.as_str())>
                                <div class=(row)>
                                    <label class="flex min-w-0 flex-1 items-center gap-3">
                                        <input
                                            type="checkbox"
                                            name="recipe_ids"
                                            value=(recipe.id.as_str())
                                            checked=""
                                        >
                                        if let Some(picture) = recipe.picture {
                                            <img class=(thumbnail) src=(picture) alt="" loading="lazy">
                                        }
                                        <span class="min-w-0 wrap-break-word">(recipe.title.as_str())</span>
                                    </label>
                                    <button
                                        class=(move_button)
                                        type="submit"
                                        name="up"
                                        value=(recipe.id.as_str())
                                        aria-label=(t!(dish_move_up, title: recipe.title.as_str()))
                                        disabled=(recipe.first.then_some(""))
                                        formnovalidate=""
                                    >
                                        "↑"
                                    </button>
                                    <button
                                        class=(move_button)
                                        type="submit"
                                        name="down"
                                        value=(recipe.id.as_str())
                                        aria-label=(t!(dish_move_down, title: recipe.title.as_str()))
                                        disabled=(recipe.last.then_some(""))
                                        formnovalidate=""
                                    >
                                        "↓"
                                    </button>
                                </div>
                                if let Some(message) = recipe.refused {
                                    <p class=(FIELD_REFUSED) data-field=(format!("recipe_ids[{at}]"))>
                                        (message)
                                    </p>
                                }
                            </li>
                        }
                    </ol>
                </fieldset>

                <fieldset>
                    <legend class=(H2)>(t!(dish_others))</legend>
                    if no_recipe {
                        <p class="mt-4 text-stone-600" data-others="no-recipe">
                            (t!(dish_no_recipe_yet))
                            " "
                            <a class=(LINK) href=(href!(recipes::new::blank))>(t!(recipes_new))</a>
                        </p>
                    } else {
                        <p class="mt-1 text-sm text-stone-500">(t!(dish_others_hint))</p>
                        <div class="mt-4 flex gap-3">
                            <label class="sr-only" for="dish-q">(t!(recipes_search_label))</label>
                            <input
                                id="dish-q"
                                class="block w-full rounded-lg border border-stone-300 bg-white px-3 py-2"
                                type="search"
                                name="q"
                                value=(typed.q)
                                placeholder=(t!(recipes_search_label))
                            >
                            // Nothing has to be filled to search.
                            <button
                                class=(BUTTON_QUIET)
                                type="submit"
                                name="search"
                                value="1"
                                formnovalidate=""
                            >
                                (t!(recipes_search))
                            </button>
                        </div>
                        if none_found {
                            <p class="mt-4 text-stone-600" data-others="none">(t!(dish_others_none))</p>
                        }
                        <ul class="mt-4 grid gap-2 sm:grid-cols-2" data-rows="others">
                            for recipe in others {
                                <li data-other=(recipe.id.as_str())>
                                    <label class=(row)>
                                        <input type="checkbox" name="recipe_ids" value=(recipe.id.as_str())>
                                        if let Some(picture) = recipe.picture {
                                            <img class=(thumbnail) src=(picture) alt="" loading="lazy">
                                        }
                                        <span class="min-w-0 wrap-break-word">(recipe.title)</span>
                                    </label>
                                </li>
                            }
                        </ul>
                        if more {
                            <p class="mt-3 text-sm text-stone-500" data-others="more">
                                (t!(dish_others_more))
                            </p>
                        }
                    }
                </fieldset>

                <div class="flex flex-wrap items-center gap-4 border-t border-stone-200 pt-6">
                    <button class=(BUTTON) type="submit">(t!(dish_save))</button>
                    <a class=(LINK) href=(back.as_str())>(t!(recipe_cancel))</a>
                </div>
            </form>
        </main>
    })
}
