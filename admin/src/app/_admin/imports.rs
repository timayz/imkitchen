//! `/imports`: each try at importing a recipe from a page of the web, all
//! of them or only those that gave no recipe, and `/imports/sources`: the
//! sites they are made from.

use topcoat::{
    Result,
    context::Cx,
    router::{href, page, query_params},
    view::{View, class, view},
};

use super::recipes::{self, recipe_id};
use super::users::{self, user_id};
use super::{H1, LINK, MAIN, TABLE, TD, TH, pager, read, when};
use crate::app::db;
use crate::imports::Filter;
use crate::listing::{PAGE_SIZE, Pages, searched};

/// How many sites the page of the sources lists.
const SOURCES_SHOWN: i64 = 500;

#[query_params]
struct Query {
    /// Only the tries that gave no recipe.
    failed: Option<String>,
    /// Only the tries from this site.
    host: Option<String>,
    page: Option<u32>,
}

/// Why a try gave no recipe, said to a person.
fn reason(reason: &str) -> &str {
    match reason {
        "page_unreachable" => "The page was not read",
        "no_recipe_on_page" => "No recipe on the page",
        "internal" => "The api failed",
        other => other,
    }
}

#[page]
pub async fn list(cx: &Cx) -> Result<impl View> {
    let db = db(cx);
    let query = query_params::<Query>(cx).ok();
    let filter = Filter {
        failed: query.is_some_and(|query| query.failed.is_some()),
        host: searched(query.and_then(|query| query.host.as_deref())),
    };
    let pages = Pages::new(
        query.and_then(|query| query.page),
        read(crate::imports::count(db, filter).await)?,
    );
    let tries = read(crate::imports::list(db, filter, pages.offset(), PAGE_SIZE).await)?;
    let people = tries.iter().map(|import| import.user_id.as_str());
    let people = read(crate::users::names(db, people).await)?;
    // The list, with the same site and the tries `failed` says.
    let listed = |failed: bool| {
        let mut pairs = Vec::new();
        if failed {
            pairs.push(("failed", "1".to_owned()));
        }
        if let Some(host) = filter.host {
            pairs.push(("host", host.to_owned()));
        }
        pairs
    };
    let at = |page: u32| {
        let mut pairs = listed(filter.failed);
        pairs.push(("page", page.to_string()));
        href!(list).query(pairs).resolve(cx)
    };
    let (previous, next) = (pages.previous().map(at), pages.next().map(at));
    let every = href!(list).query(listed(false)).resolve(cx);
    let failed = href!(list).query(listed(true)).resolve(cx);
    let tab = "rounded-lg border px-3 py-1";

    Ok(view! {
        <main class=(MAIN)>
            <h1 class=(H1)>"Imports"</h1>
            if let Some(host) = filter.host {
                <p class="mt-2 text-stone-600" data-host=(host)>
                    "From " (host) " · "
                    <a class=(LINK) href=(href!(list))>"every site"</a>
                </p>
            }
            <nav class="mt-6 flex flex-wrap gap-2 text-sm" aria-label="Which imports">
                <a
                    class=(class!(tab, "border-stone-900 font-semibold" if !filter.failed, "border-stone-300" if filter.failed))
                    href=(every)
                >
                    "All"
                </a>
                <a
                    class=(class!(tab, "border-stone-900 font-semibold" if filter.failed, "border-stone-300" if !filter.failed))
                    href=(failed)
                >
                    "Failed"
                </a>
                <a class=(class!(tab, "border-stone-300")) href=(href!(sources))>"Sources"</a>
            </nav>
            <table class=(TABLE)>
                <thead>
                    <tr>
                        <th class=(TH)>"When"</th>
                        <th class=(TH)>"Page"</th>
                        <th class=(TH)>"By"</th>
                        <th class=(TH)>"Outcome"</th>
                    </tr>
                </thead>
                <tbody>
                    for import in &tries {
                        <tr data-import=(if import.failed() { "failed" } else { "imported" })>
                            <td class=(TD)>(when(import.at))</td>
                            <td class=(class!(TD, "break-all"))>
                                <a class=(LINK) href=(import.url.as_str()) rel="noreferrer noopener">
                                    (import.url.as_str())
                                </a>
                                if import.page_sent {
                                    <span class="block text-stone-500">"Sent by the app: the site refuses the api"</span>
                                }
                            </td>
                            <td class=(TD)>
                                <a class=(LINK) href=(href!(users::show, user_id: import.user_id.as_str()))>
                                    (people.get(&import.user_id).map_or(import.user_id.as_str(), String::as_str))
                                </a>
                            </td>
                            <td class=(TD)>
                                if let Some(recipe_id) = &import.recipe_id {
                                    <a class=(LINK) href=(href!(recipes::show, recipe_id: recipe_id.as_str()))>
                                        "The recipe"
                                    </a>
                                } else {
                                    <span class="text-red-800" data-reason=(import.reason.as_deref().unwrap_or_default())>
                                        (reason(import.reason.as_deref().unwrap_or_default()))
                                    </span>
                                }
                            </td>
                        </tr>
                    }
                </tbody>
            </table>
            if tries.is_empty() {
                <p class="mt-6 text-stone-600" data-empty="">"No import is found."</p>
            }
            pager(pages: pages, previous: previous, next: next)
        </main>
    })
}

#[page("./sources")]
pub async fn sources(cx: &Cx) -> Result<impl View> {
    let sites = read(crate::imports::sources(db(cx), SOURCES_SHOWN).await)?;

    Ok(view! {
        <main class=(MAIN)>
            <p class="text-sm"><a class=(LINK) href=(href!(list))>"Imports"</a></p>
            <h1 class=(H1)>"Sources"</h1>
            <p class="mt-2 text-stone-600">
                "The sites recipes are imported from, the most tried first."
            </p>
            <table class=(TABLE)>
                <thead>
                    <tr>
                        <th class=(TH)>"Site"</th>
                        <th class=(TH)>"Imported"</th>
                        <th class=(TH)>"Failed"</th>
                        <th class=(TH)>"Sent by the app"</th>
                        <th class=(TH)>"Last try"</th>
                    </tr>
                </thead>
                <tbody>
                    for source in &sites {
                        <tr
                            data-source=(source.host.as_str())
                            data-imported=(source.imported)
                            data-failed=(source.failed)
                        >
                            <td class=(TD)>
                                <a class=(LINK) href=(href!(list).query([("host", source.host.as_str())]))>
                                    (source.host.as_str())
                                </a>
                            </td>
                            <td class=(TD)>(source.imported)</td>
                            <td class=(TD)>
                                if source.failed > 0 {
                                    <a
                                        class=(LINK)
                                        href=(href!(list).query([("failed", "1"), ("host", source.host.as_str())]))
                                    >
                                        (source.failed)
                                    </a>
                                } else {
                                    "0"
                                }
                            </td>
                            <td class=(TD)>(source.page_sent)</td>
                            <td class=(TD)>(when(source.last_at))</td>
                        </tr>
                    }
                </tbody>
            </table>
            if sites.is_empty() {
                <p class="mt-6 text-stone-600" data-empty="">"Nothing was imported yet."</p>
            }
        </main>
    })
}
