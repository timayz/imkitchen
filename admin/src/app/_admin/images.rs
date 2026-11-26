//! `/images`: the pictures of the recipes, all that are used or only those
//! several recipes share, and `/images/{image_id}`: one of them and the
//! recipes that show it.

use topcoat::{
    Result,
    context::Cx,
    router::{error::not_found, href, page, path_param, query_params},
    view::{View, class, view},
};

use super::recipes::{self, recipe_id};
use super::users::{self, user_id};
use super::{H1, H2, LINK, MAIN, TABLE, TD, TH, pager, read, when};
use crate::app::{api, db};
use crate::listing::{PAGE_SIZE, Pages};

/// How many recipes the page of a picture lists.
const RECIPES_SHOWN: i64 = 200;

path_param!(pub image_id);

#[query_params]
struct Query {
    /// Only the pictures used more than once.
    shared: Option<String>,
    page: Option<u32>,
}

/// A weight in bytes, as a person reads it.
fn weight(bytes: i64) -> String {
    if bytes < 1024 * 1024 {
        format!("{} kB", (bytes + 1023) / 1024)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[page]
pub async fn list(cx: &Cx) -> Result<impl View> {
    let db = db(cx);
    let query = query_params::<Query>(cx).ok();
    let shared = query.is_some_and(|query| query.shared.is_some());
    let pages = Pages::new(
        query.and_then(|query| query.page),
        read(crate::images::count(db, shared).await)?,
    );
    let images = read(crate::images::list(db, shared, pages.offset(), PAGE_SIZE).await)?;
    let listed = |shared: bool| shared.then_some(("shared", "1".to_owned()));
    let at = |page: u32| {
        let pairs: Vec<_> = listed(shared)
            .into_iter()
            .chain([("page", page.to_string())])
            .collect();
        href!(list).query(pairs).resolve(cx)
    };
    let (previous, next) = (pages.previous().map(at), pages.next().map(at));
    let tab = "rounded-lg border px-3 py-1";

    Ok(view! {
        <main class=(MAIN)>
            <h1 class=(H1)>"Pictures"</h1>
            <p class="mt-2 text-stone-600">
                "A picture is stored once however many recipes show it."
            </p>
            <nav class="mt-6 flex flex-wrap gap-2 text-sm" aria-label="Which pictures">
                <a
                    class=(class!(tab, "border-stone-900 font-semibold" if !shared, "border-stone-300" if shared))
                    href=(href!(list))
                >
                    "All"
                </a>
                <a
                    class=(class!(tab, "border-stone-900 font-semibold" if shared, "border-stone-300" if !shared))
                    href=(href!(list).query([("shared", "1")]))
                >
                    "Shared"
                </a>
            </nav>
            <table class=(TABLE)>
                <thead>
                    <tr>
                        <th class=(TH)>"Picture"</th>
                        <th class=(TH)>"Uses"</th>
                        <th class=(TH)>"Size"</th>
                        <th class=(TH)>"Weight"</th>
                        <th class=(TH)>"Stored"</th>
                    </tr>
                </thead>
                <tbody>
                    for image in &images {
                        <tr data-image=(image.image_id.as_str()) data-uses=(image.uses)>
                            <td class=(TD)>
                                <a class=(LINK) href=(href!(show, image_id: image.image_id.as_str()))>
                                    if let Some(picture) = api(cx).picture(&image.file_id) {
                                        <img class="h-16 w-16 rounded object-cover" src=(picture) alt="" loading="lazy">
                                    } else {
                                        (image.image_id.get(..12).unwrap_or(&image.image_id))
                                    }
                                </a>
                            </td>
                            <td class=(TD)>(image.uses)</td>
                            <td class=(TD)>(image.width) " × " (image.height)</td>
                            <td class=(TD)>(weight(image.bytes))</td>
                            <td class=(TD)>(when(image.stored_at))</td>
                        </tr>
                    }
                </tbody>
            </table>
            if images.is_empty() {
                <p class="mt-6 text-stone-600" data-empty="">"No picture is found."</p>
            }
            pager(pages: pages, previous: previous, next: next)
        </main>
    })
}

#[page("./{image_id}")]
pub async fn show(cx: &Cx) -> Result<impl View> {
    let db = db(cx);
    let Some(image) = read(crate::images::get(db, image_id(cx)).await)? else {
        return Err(not_found().into());
    };
    let shown_by = read(crate::recipes::showing(db, &image.image_id, RECIPES_SHOWN).await)?;
    let owners = shown_by.iter().map(|recipe| recipe.owner_id.as_str());
    let owners = read(crate::users::names(db, owners).await)?;

    Ok(view! {
        <main class=(MAIN) data-image=(image.image_id.as_str())>
            <p class="text-sm"><a class=(LINK) href=(href!(list))>"Pictures"</a></p>
            <h1 class=(H1)>"Picture"</h1>
            if let Some(picture) = api(cx).picture(&image.file_id) {
                <img class="mt-6 max-h-80 rounded-lg" src=(picture) alt="" loading="lazy">
            }
            <dl class="mt-6 grid gap-x-8 gap-y-3 text-sm sm:grid-cols-2">
                <div><dt class="text-stone-500">"Uses"</dt><dd data-uses=(image.uses)>(image.uses)</dd></div>
                <div><dt class="text-stone-500">"Size"</dt><dd>(image.width) " × " (image.height)</dd></div>
                <div><dt class="text-stone-500">"Weight"</dt><dd>(weight(image.bytes))</dd></div>
                <div><dt class="text-stone-500">"Stored"</dt><dd>(when(image.stored_at))</dd></div>
                <div class="sm:col-span-2">
                    <dt class="text-stone-500">"SHA-256"</dt>
                    <dd class="break-all">(image.image_id.as_str())</dd>
                </div>
            </dl>

            <h2 class=(H2)>"Shown by"</h2>
            if shown_by.is_empty() {
                <p class="mt-4 text-sm text-stone-600">"No recipe."</p>
            } else {
                <ul class="mt-4 grid gap-1 text-sm" data-shown-by=(shown_by.len())>
                    for recipe in &shown_by {
                        <li>
                            <a class=(LINK) href=(href!(recipes::show, recipe_id: recipe.id.as_str()))>
                                (recipe.title.as_str())
                            </a>
                            " of "
                            <a class=(LINK) href=(href!(users::show, user_id: recipe.owner_id.as_str()))>
                                (owners.get(&recipe.owner_id).map_or(recipe.owner_id.as_str(), String::as_str))
                            </a>
                        </li>
                    }
                </ul>
            }
        </main>
    })
}
