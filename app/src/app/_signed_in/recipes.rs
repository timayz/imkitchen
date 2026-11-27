//! `/recipes`: the recipes of who reads the page, searched by title or by
//! ingredient, and `/recipes/{recipe_id}`: one of them, all it says, and
//! the button that deletes it. A recipe is written at `/recipes/new`
//! ([`new`]), imported from a page of the web at `/recipes/import`
//! ([`import`]) and rewritten at `/recipes/{recipe_id}/edit` ([`_form`]).
//! Once it is there, nothing tells one that was imported from one that was
//! written.
//!
//! Someone else's recipe is one that does not exist.

pub mod _form;
pub mod import;
pub mod new;

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

use super::{
    BUTTON, BUTTON_QUIET, Failure, H1, H2, LINK, MAIN, account, asking, failed, followed, read,
};
use crate::app::{api, db, language, t};
use crate::recipes::{Ingredient, Instruction, Recipe};

path_param!(pub recipe_id);

/// How many recipes a page of the list shows.
const PAGE: usize = 24;

/// What the address of the list says of a recipe that was just deleted.
const DELETED: &str = "deleted";

#[query_params]
struct Query {
    q: Option<String>,
    /// Where the page before stopped.
    after: Option<String>,
    /// What was just done to a recipe.
    done: Option<String>,
}

/// A recipe as the api tells of it, in answer to the command that wrote it
/// or when asked: what app waits to have followed before it shows the
/// recipe. No page is read from it.
#[derive(Debug, Deserialize)]
pub(crate) struct Told {
    pub(crate) id: String,
    title: String,
    #[serde(default)]
    description: String,
    ingredients: Vec<Ingredient>,
    instructions: Vec<Instruction>,
    prep_minutes: i64,
    cook_minutes: i64,
    servings: i64,
    #[serde(default)]
    image: Option<ToldImage>,
}

#[derive(Debug, Deserialize)]
struct ToldImage {
    large: ToldFile,
}

#[derive(Debug, Deserialize)]
struct ToldFile {
    /// `/v1/images/1200/{file_id}.webp`.
    url: String,
}

impl Told {
    /// What the files of its picture are named after, when it shows one.
    fn file_id(&self) -> Option<&str> {
        let file = self.image.as_ref()?.large.url.rsplit('/').next()?;
        file.strip_suffix(".webp")
    }

    /// Whether `recipe`, as app followed it, says the same.
    fn is(&self, recipe: &Recipe) -> bool {
        self.title == recipe.title
            && self.description == recipe.description
            && self.ingredients == recipe.ingredients.0
            && self.instructions == recipe.instructions.0
            && self.prep_minutes == recipe.prep_minutes
            && self.cook_minutes == recipe.cook_minutes
            && self.servings == recipe.servings
            && self.file_id() == recipe.file_id.as_deref()
    }
}

/// Waits until app followed the recipe as far as the api tells of it, for
/// the page the browser is sent to to show it as it was just left.
pub(crate) async fn caught_up(cx: &Cx, told: &Told) {
    let (db, owner_id) = (db(cx), account(cx).id.as_str());
    followed(|| async {
        let recipe = crate::recipes::get_own(db, owner_id, &told.id).await?;
        Ok(recipe.is_some_and(|recipe| told.is(&recipe)))
    })
    .await;
}

/// Waits until app followed the recipe `id` as far as the api has written
/// it. An api that does not say is not waited for: the page shows the
/// recipe a moment later.
pub(crate) async fn settled(cx: &Cx, id: &str) {
    let language = language(cx);
    let told = api(cx)
        .read::<Told>(asking(cx, &language), &["recipes", id])
        .await;
    if let Ok(told) = told {
        caught_up(cx, &told).await;
    }
}

/// An ingredient as a recipe writes it: "200 g flour", "2 eggs", "salt".
fn line(ingredient: &Ingredient) -> String {
    [
        ingredient.quantity.map(|quantity| quantity.to_string()),
        ingredient.unit.clone(),
        Some(ingredient.name.clone()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
}

/// A recipe as the list shows it.
struct Card {
    id: String,
    address: String,
    title: String,
    picture: Option<String>,
    /// From the first thing done to the table: 0 when the recipe says no
    /// time.
    minutes: i64,
    servings: i64,
}

#[page(with(Metadata::new().title(t!(recipes_meta_title))))]
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
    let mut recipes = read(
        crate::recipes::list_of(db(cx), &account(cx).id, q, after.as_ref(), PAGE as i64 + 1).await,
    )?;
    let more = recipes.len() > PAGE;
    recipes.truncate(PAGE);
    let next = recipes.last().filter(|_| more).map(|last| {
        href!(list)
            .query([("q", q.unwrap_or_default())])
            .query([("after", last.cursor().param())])
            .resolve(cx)
            .to_string()
    });
    let cards: Vec<Card> = recipes
        .into_iter()
        .map(|recipe| Card {
            address: href!(show, recipe_id: recipe.id.as_str())
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
    // The first page of everything: the list as it opens.
    let whole = q.is_none() && after.is_none();
    let searched = q.unwrap_or_default().to_owned();

    Ok(view! {
        <main class="mx-auto max-w-5xl px-4 py-8 md:px-8 md:py-12">
            <div class="flex flex-wrap items-center justify-between gap-4">
                <h1 class=(H1)>(t!(recipes_title))</h1>
                <p class="flex gap-3 text-sm">
                    <a class=(BUTTON_QUIET) href=(href!(import::ask)) data-action="import">
                        (t!(recipes_import))
                    </a>
                    <a class=(BUTTON) href=(href!(new::blank)) data-action="new">
                        (t!(recipes_new))
                    </a>
                </p>
            </div>
            if deleted {
                <p
                    class="mt-6 rounded-lg border border-green-300 bg-green-50 px-4 py-3 text-green-900"
                    role="status"
                    data-done=(DELETED)
                >
                    (t!(recipe_deleted))
                </p>
            }
            <form class="mt-6 flex gap-3" method="get" action=(href!(list)) role="search">
                <label class="sr-only" for="recipes-q">(t!(recipes_search_label))</label>
                <input
                    id="recipes-q"
                    class="block w-full rounded-lg border border-stone-300 bg-white px-3 py-2"
                    type="search"
                    name="q"
                    value=(searched)
                    placeholder=(t!(recipes_search_label))
                >
                <button class=(BUTTON_QUIET) type="submit">(t!(recipes_search))</button>
            </form>
            if cards.is_empty() {
                <p class="mt-8 text-stone-600" data-empty="">
                    if whole {
                        (t!(recipes_none))
                    } else {
                        (t!(recipes_none_found))
                    }
                </p>
            }
            <ul class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                for card in cards {
                    <li data-recipe=(card.id)>
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
                                <h2 class="font-semibold">(card.title)</h2>
                                <p
                                    class="mt-1 text-sm text-stone-500"
                                    data-minutes=(card.minutes)
                                    data-servings=(card.servings)
                                >
                                    if card.minutes > 0 {
                                        (t!(recipe_minutes, minutes: card.minutes))
                                        " · "
                                    }
                                    (t!(recipe_serves, count: card.servings))
                                </p>
                            </div>
                        </a>
                    </li>
                }
            </ul>
            if let Some(next) = next {
                <p class="mt-8 text-center">
                    <a class=(BUTTON_QUIET) href=(next) rel="next" data-more="">(t!(recipes_more))</a>
                </p>
            }
        </main>
    })
}

#[page("./{recipe_id}", with(Metadata::new().title(t!(recipes_meta_title))))]
pub async fn show(cx: &Cx) -> Result<impl View> {
    let id = recipe_id(cx);
    let Some(recipe) = read(crate::recipes::get_own(db(cx), &account(cx).id, id).await)? else {
        return Err(not_found().into());
    };
    let picture = recipe.file_id.as_deref().map(|file_id| {
        (
            api(cx).picture(file_id, Rendition::Page),
            recipe.image_width,
            recipe.image_height,
        )
    });
    let ingredients: Vec<String> = recipe.ingredients.0.iter().map(line).collect();
    let steps = recipe.instructions.0;

    Ok(view! {
        <main class=(MAIN) data-recipe=(recipe.id)>
            <p class="text-sm"><a class=(LINK) href=(href!(list))>(t!(recipes_title))</a></p>
            // A title someone typed, or a site wrote: it may be one long
            // word.
            <h1 class=(class!(H1, "mt-2 wrap-break-word"))>(recipe.title)</h1>
            if let Some((picture, width, height)) = picture {
                <img
                    class="mt-6 h-auto w-full rounded-xl"
                    src=(picture)
                    width=(width)
                    height=(height)
                    alt=""
                    data-picture=""
                >
            }
            if !recipe.description.is_empty() {
                <p class="mt-6 whitespace-pre-line text-lg leading-relaxed text-stone-600">
                    (recipe.description)
                </p>
            }
            <dl class="mt-6 flex flex-wrap gap-x-10 gap-y-3 text-sm">
                if recipe.prep_minutes > 0 {
                    <div>
                        <dt class="text-stone-500">(t!(recipe_prep))</dt>
                        <dd data-prep=(recipe.prep_minutes)>
                            (t!(recipe_minutes, minutes: recipe.prep_minutes))
                        </dd>
                    </div>
                }
                if recipe.cook_minutes > 0 {
                    <div>
                        <dt class="text-stone-500">(t!(recipe_cook))</dt>
                        <dd data-cook=(recipe.cook_minutes)>
                            (t!(recipe_minutes, minutes: recipe.cook_minutes))
                        </dd>
                    </div>
                }
                <div>
                    <dt class="text-stone-500">(t!(recipe_servings))</dt>
                    <dd data-servings=(recipe.servings)>(recipe.servings)</dd>
                </div>
            </dl>

            <h2 class=(H2)>(t!(recipe_ingredients))</h2>
            <ul class="mt-4 list-disc space-y-1 pl-6" data-ingredients=(ingredients.len())>
                for ingredient in ingredients {
                    <li>(ingredient)</li>
                }
            </ul>

            <h2 class=(H2)>(t!(recipe_instructions))</h2>
            <ol class="mt-4 list-decimal space-y-3 pl-6" data-instructions=(steps.len())>
                for step in steps {
                    <li class="whitespace-pre-line">
                        (step.text)
                        if let Some(minutes) = step.timer_minutes {
                            <span class="ml-2 text-sm text-stone-500" data-timer=(minutes)>
                                (t!(recipe_minutes, minutes: minutes))
                            </span>
                        }
                    </li>
                }
            </ol>

            <div class="mt-12 flex flex-wrap items-start gap-3 border-t border-stone-200 pt-6 text-sm">
                <a
                    class=(BUTTON_QUIET)
                    href=(href!(_form::edit, recipe_id: id))
                    data-action="edit"
                >
                    (t!(recipe_edit))
                </a>
                <details class="rounded-lg border border-stone-300 bg-white px-5 py-3">
                    <summary class="cursor-pointer font-semibold text-red-800">
                        (t!(recipe_delete))
                    </summary>
                    <p class="mt-3 max-w-sm text-stone-700">(t!(recipe_delete_warning))</p>
                    <form
                        class="mt-3"
                        method="post"
                        action=(href!(delete, recipe_id: id))
                        data-action="delete"
                    >
                        <button
                            class="rounded-lg bg-red-700 px-4 py-2 font-semibold text-white hover:bg-red-800"
                            type="submit"
                        >
                            (t!(recipe_delete_confirm))
                        </button>
                    </form>
                </details>
            </div>
        </main>
    })
}

/// Deletes the recipe, and sends the browser to the list once app followed
/// that it is gone.
#[page(POST "./{recipe_id}/delete", with(Metadata::new().title(t!(recipes_meta_title))))]
async fn delete(cx: &Cx) -> Result<impl View> {
    let id = recipe_id(cx);
    let (db, owner_id) = (db(cx), account(cx).id.as_str());
    let language = language(cx);
    let deleted = api(cx)
        .command(
            asking(cx, &language),
            Method::DELETE,
            &["recipes", id],
            None::<&()>,
        )
        .await;
    let failure = match deleted {
        Ok(()) => {
            followed(|| async { Ok(crate::recipes::get_own(db, owner_id, id).await?.is_none()) })
                .await;
            let then = href!(list).query([("done", DELETED)]).resolve(cx);
            return Err(see_other(then).into());
        }
        Err(err) => Failure::of(cx, err)?,
    };

    Ok(view! {
        <main class=(MAIN)>
            <h1 class=(H1)>(t!(recipe_not_deleted))</h1>
            <div class="mt-6">failed(failure: Some(failure))</div>
            <p class="mt-6">
                <a class=(LINK) href=(href!(show, recipe_id: id))>(t!(recipe_back))</a>
            </p>
        </main>
    })
}
