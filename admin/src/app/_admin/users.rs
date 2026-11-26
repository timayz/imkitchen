//! `/users`: who has an account, searched by address or by name, and
//! `/users/{user_id}`: one of them, their sessions and their recipes.

use topcoat::{
    Result,
    context::Cx,
    router::{error::not_found, href, page, path_param, query_params},
    view::{View, view},
};

use super::recipes::{self, recipe_id};
use super::{H1, H2, LINK, MAIN, TABLE, TD, TH, pager, read, search, when};
use crate::app::db;
use crate::listing::{PAGE_SIZE, Pages, searched};

/// How many recipes the page of a user lists.
const RECIPES_SHOWN: i64 = 20;

path_param!(pub user_id);

#[query_params]
struct Query {
    q: Option<String>,
    page: Option<u32>,
}

#[page]
pub async fn list(cx: &Cx) -> Result<impl View> {
    let db = db(cx);
    let query = query_params::<Query>(cx).ok();
    let q = searched(query.and_then(|query| query.q.as_deref()));
    let pages = Pages::new(
        query.and_then(|query| query.page),
        read(crate::users::found(db, q).await)?,
    );
    let users = read(crate::users::list(db, q, pages.offset(), PAGE_SIZE).await)?;
    let at = |page: u32| {
        href!(list)
            .query([("q", q.unwrap_or_default())])
            .query([("page", page)])
            .resolve(cx)
    };
    let (previous, next) = (pages.previous().map(at), pages.next().map(at));

    Ok(view! {
        <main class=(MAIN)>
            <h1 class=(H1)>"Users"</h1>
            search(
                action: href!(list).resolve(cx),
                q: q.unwrap_or_default().to_owned(),
                label: "Address or name",
            )
            <table class=(TABLE)>
                <thead>
                    <tr>
                        <th class=(TH)>"Name"</th>
                        <th class=(TH)>"Address"</th>
                        <th class=(TH)>"Role"</th>
                        <th class=(TH)>"Registered"</th>
                        <th class=(TH)>"Last sign-in"</th>
                    </tr>
                </thead>
                <tbody>
                    for user in &users {
                        <tr data-user=(user.id.as_str())>
                            <td class=(TD)>
                                <a class=(LINK) href=(href!(show, user_id: user.id.as_str()))>
                                    (user.label.as_str())
                                </a>
                            </td>
                            <td class=(TD)>(user.account.as_str())</td>
                            <td class=(TD)>(role(user.is_admin, user.deleted))</td>
                            <td class=(TD)>(when(user.registered_at))</td>
                            <td class=(TD)>(user.last_sign_in_at.map(when).unwrap_or_default())</td>
                        </tr>
                    }
                </tbody>
            </table>
            if users.is_empty() {
                <p class="mt-6 text-stone-600" data-empty="">"No user is found."</p>
            }
            pager(pages: pages, previous: previous, next: next)
        </main>
    })
}

/// What a user is to imkitchen, in a word.
fn role(is_admin: bool, deleted: bool) -> &'static str {
    match (deleted, is_admin) {
        (true, _) => "Deleted",
        (false, true) => "Administrator",
        (false, false) => "User",
    }
}

#[page("./{user_id}")]
pub async fn show(cx: &Cx) -> Result<impl View> {
    let db = db(cx);
    let id = user_id(cx);
    let Some(user) = read(crate::users::get(db, id).await)? else {
        return Err(not_found().into());
    };
    let sessions = read(crate::users::sessions(db, id).await)?;
    let recipe_count = read(crate::recipes::count_of(db, id).await)?;
    let recipes = read(crate::recipes::list_of(db, id, RECIPES_SHOWN).await)?;
    let now = timada_core::unix_now() as i64;
    let methods = [
        (user.has_password, "password"),
        (user.oauth2_count > 0, "another site"),
        (user.passkey_count > 0, "passkey"),
    ]
    .into_iter()
    .filter_map(|(has, method)| has.then_some(method))
    .collect::<Vec<_>>()
    .join(", ");

    Ok(view! {
        <main class=(MAIN) data-user=(user.id.as_str())>
            <p class="text-sm"><a class=(LINK) href=(href!(list))>"Users"</a></p>
            <h1 class=(H1)>(user.label.as_str())</h1>
            <dl class="mt-6 grid gap-x-8 gap-y-3 text-sm sm:grid-cols-2">
                <div><dt class="text-stone-500">"Address"</dt><dd>(user.account.as_str())</dd></div>
                <div>
                    <dt class="text-stone-500">"Role"</dt>
                    <dd data-role="">(role(user.is_admin, user.deleted))</dd>
                </div>
                <div><dt class="text-stone-500">"Registered"</dt><dd>(when(user.registered_at))</dd></div>
                <div>
                    <dt class="text-stone-500">"Last sign-in"</dt>
                    <dd>(user.last_sign_in_at.map(when).unwrap_or_else(|| "Never".to_owned()))</dd>
                </div>
                <div><dt class="text-stone-500">"Sign-ins"</dt><dd>(user.sign_in_count)</dd></div>
                <div><dt class="text-stone-500">"Signs in with"</dt><dd>(methods)</dd></div>
                <div><dt class="text-stone-500">"Id"</dt><dd class="break-all">(user.id.as_str())</dd></div>
            </dl>

            <h2 class=(H2)>"Recipes (" (recipe_count) ")"</h2>
            if recipes.is_empty() {
                <p class="mt-4 text-sm text-stone-600">"None."</p>
            } else {
                <ul class="mt-4 grid gap-1 text-sm" data-recipes=(recipe_count)>
                    for recipe in &recipes {
                        <li>
                            <a class=(LINK) href=(href!(recipes::show, recipe_id: recipe.id.as_str()))>
                                (recipe.title.as_str())
                            </a>
                            <span class="text-stone-500">" · " (when(recipe.created_at))</span>
                        </li>
                    }
                </ul>
            }

            <h2 class=(H2)>"Sessions"</h2>
            if sessions.is_empty() {
                <p class="mt-4 text-sm text-stone-600">"None."</p>
            } else {
                <table class=(TABLE)>
                    <thead>
                        <tr>
                            <th class=(TH)>"Started"</th>
                            <th class=(TH)>"Until"</th>
                            <th class=(TH)>"State"</th>
                            <th class=(TH)>"Browser"</th>
                        </tr>
                    </thead>
                    <tbody>
                        for session in &sessions {
                            <tr data-session=(if session.is_active_at(now) { "active" } else { "ended" })>
                                <td class=(TD)>(when(session.started_at))</td>
                                <td class=(TD)>(when(session.expires_at))</td>
                                <td class=(TD)>(if session.is_active_at(now) { "Active" } else { "Ended" })</td>
                                <td class=(TD)>(session.user_agent.as_deref().unwrap_or_default())</td>
                            </tr>
                        }
                    </tbody>
                </table>
            }
        </main>
    })
}
