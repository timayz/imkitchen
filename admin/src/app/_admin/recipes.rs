//! `/recipes`: every recipe, searched by title or by ingredient, and
//! `/recipes/{recipe_id}`: one of them, all it says, and the button that
//! deletes it.

use topcoat::{
    Result,
    context::Cx,
    router::{error::not_found, href, page, path_param, query_params},
    view::{View, view},
};

use super::images::{self, image_id};
use super::users::{self, user_id};
use super::{
    ACTIONS, DONE, H1, H2, LINK, MAIN, TABLE, TD, TH, act_for_good, done, fresh, not_done, notice,
    pager, read, search, token, when,
};
use crate::app::{api, db, events};
use crate::listing::{PAGE_SIZE, Pages, searched};
use crate::recipes::Ingredient;

path_param!(pub recipe_id);

#[query_params]
struct Query {
    q: Option<String>,
    page: Option<u32>,
    /// What was just done to a recipe.
    done: Option<String>,
    /// The recipe that was just deleted, which the table may hold a moment
    /// longer: the list leaves it out.
    gone: Option<String>,
}

#[page]
pub async fn list(cx: &Cx) -> Result<impl View> {
    let db = db(cx);
    let query = query_params::<Query>(cx).ok();
    let q = searched(query.and_then(|query| query.q.as_deref()));
    let deleted = query.is_some_and(|query| query.done.as_deref() == Some("deleted"));
    let gone = query.and_then(|query| query.gone.as_deref());
    let pages = Pages::new(
        query.and_then(|query| query.page),
        read(crate::recipes::count(db, q).await)?,
    );
    let mut recipes = read(crate::recipes::list(db, q, pages.offset(), PAGE_SIZE).await)?;
    recipes.retain(|recipe| Some(recipe.id.as_str()) != gone);
    let owners = recipes.iter().map(|recipe| recipe.owner_id.as_str());
    let owners = read(crate::users::names(db, owners).await)?;
    let at = |page: u32| {
        href!(list)
            .query([("q", q.unwrap_or_default())])
            .query([("page", page)])
            .resolve(cx)
    };
    let (previous, next) = (pages.previous().map(at), pages.next().map(at));

    Ok(view! {
        <main class=(MAIN)>
            <h1 class=(H1)>"Recipes"</h1>
            if deleted {
                notice(what: "deleted", message: "The recipe is deleted.")
            }
            search(
                action: href!(list).resolve(cx),
                q: q.unwrap_or_default().to_owned(),
                label: "Title or ingredient",
            )
            <table class=(TABLE)>
                <thead>
                    <tr>
                        <th class=(TH)>"Title"</th>
                        <th class=(TH)>"Of"</th>
                        <th class=(TH)>"Picture"</th>
                        <th class=(TH)>"Written"</th>
                    </tr>
                </thead>
                <tbody>
                    for recipe in &recipes {
                        <tr data-recipe=(recipe.id.as_str())>
                            <td class=(TD)>
                                <a class=(LINK) href=(href!(show, recipe_id: recipe.id.as_str()))>
                                    (recipe.title.as_str())
                                </a>
                            </td>
                            <td class=(TD)>
                                <a class=(LINK) href=(href!(users::show, user_id: recipe.owner_id.as_str()))>
                                    (owners.get(&recipe.owner_id).map_or(recipe.owner_id.as_str(), String::as_str))
                                </a>
                            </td>
                            <td class=(TD)>(if recipe.file_id.is_some() { "Yes" } else { "" })</td>
                            <td class=(TD)>(when(recipe.created_at))</td>
                        </tr>
                    }
                </tbody>
            </table>
            if recipes.is_empty() {
                <p class="mt-6 text-stone-600" data-empty="">"No recipe is found."</p>
            }
            pager(pages: pages, previous: previous, next: next)
        </main>
    })
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

#[page("./{recipe_id}")]
pub async fn show(cx: &Cx) -> Result<impl View> {
    let (db, id) = (db(cx), recipe_id(cx));
    let recipe = fresh(
        cx,
        crate::recipes::get(events(cx), db, id),
        crate::recipes::row(db, id),
    )
    .await?;
    let Some(recipe) = recipe else {
        return Err(not_found().into());
    };
    let owner = read(crate::users::get(db, &recipe.owner_id).await)?;
    let owner = owner.map_or(recipe.owner_id.clone(), |owner| owner.label);
    let import = read(crate::imports::of_recipe(db, &recipe.id).await)?;
    let picture = recipe
        .file_id
        .as_deref()
        .and_then(|file_id| api(cx).picture(file_id));

    Ok(view! {
        <main class=(MAIN) data-recipe=(recipe.id.as_str())>
            <p class="text-sm"><a class=(LINK) href=(href!(list))>"Recipes"</a></p>
            <h1 class=(H1)>(recipe.title.as_str())</h1>
            <dl class="mt-6 grid gap-x-8 gap-y-3 text-sm sm:grid-cols-2">
                <div>
                    <dt class="text-stone-500">"Of"</dt>
                    <dd>
                        <a class=(LINK) href=(href!(users::show, user_id: recipe.owner_id.as_str()))>
                            (owner)
                        </a>
                    </dd>
                </div>
                <div><dt class="text-stone-500">"Serves"</dt><dd>(recipe.servings)</dd></div>
                <div>
                    <dt class="text-stone-500">"Preparation"</dt>
                    <dd>(recipe.prep_minutes) " min"</dd>
                </div>
                <div><dt class="text-stone-500">"Cooking"</dt><dd>(recipe.cook_minutes) " min"</dd></div>
                <div><dt class="text-stone-500">"Written"</dt><dd>(when(recipe.created_at))</dd></div>
                <div><dt class="text-stone-500">"Changed"</dt><dd>(when(recipe.updated_at))</dd></div>
                if let Some(import) = &import {
                    <div class="sm:col-span-2">
                        <dt class="text-stone-500">"Imported from"</dt>
                        <dd class="break-all" data-imported-from=(import.host.as_str())>
                            <a class=(LINK) href=(import.url.as_str()) rel="noreferrer noopener">
                                (import.url.as_str())
                            </a>
                        </dd>
                    </div>
                }
                if let Some(image_id) = &recipe.image_id {
                    <div class="sm:col-span-2">
                        <dt class="text-stone-500">"Picture"</dt>
                        <dd>
                            <a class=(LINK) href=(href!(images::show, image_id: image_id.as_str()))>
                                if let Some(picture) = &picture {
                                    <img class="mt-1 max-h-48 rounded-lg" src=(picture.as_str()) alt="" loading="lazy">
                                } else {
                                    "See the picture"
                                }
                            </a>
                        </dd>
                    </div>
                }
            </dl>
            if !recipe.description.is_empty() {
                <p class="mt-6 whitespace-pre-line text-stone-700">(recipe.description.as_str())</p>
            }

            <div class=(ACTIONS)>
                act_for_good(
                    action: href!(delete, recipe_id: recipe.id.as_str()).resolve(cx),
                    what: "delete",
                    label: "Delete the recipe",
                    warning: "The recipe is gone for good, for its owner too: it leaves their plats and their list, and its picture goes unless another recipe shows it.",
                )
            </div>

            <h2 class=(H2)>"Ingredients"</h2>
            <ul class="mt-4 list-disc pl-6 text-sm" data-ingredients=(recipe.ingredients.0.len())>
                for ingredient in &recipe.ingredients.0 {
                    <li>(line(ingredient))</li>
                }
            </ul>

            <h2 class=(H2)>"Steps"</h2>
            <ol class="mt-4 list-decimal pl-6 text-sm" data-instructions=(recipe.instructions.0.len())>
                for step in &recipe.instructions.0 {
                    <li class="mt-1 whitespace-pre-line">
                        (step.text.as_str())
                        if let Some(minutes) = step.timer_minutes {
                            <span class="ml-2 text-stone-500" data-timer=(minutes)>(format!("{minutes} min"))</span>
                        }
                    </li>
                }
            </ol>
        </main>
    })
}

/// Deletes the recipe, and sends the browser to the list, which leaves it
/// out while its row is still there.
#[page(POST "./{recipe_id}/delete")]
async fn delete(cx: &Cx) -> Result<impl View> {
    let id = recipe_id(cx);
    let outcome = api(cx).delete_recipe(&token(cx)?, id).await;
    let then = href!(list)
        .query([(DONE, "deleted")])
        .query([("gone", id)])
        .resolve(cx);
    let failure = done(cx, outcome, then)?;
    Ok(view! { not_done(failure: failure, back: href!(show, recipe_id: id).resolve(cx)) })
}
