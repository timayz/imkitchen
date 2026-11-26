//! `/users`: who has an account, searched by address or by name, and
//! `/users/{user_id}`: one of them, their sessions and their recipes, and
//! what an administrator does with their account: suspending it,
//! reactivating it, deleting it.

use timada_core::ROOT_USER_ID;
use topcoat::{
    Result,
    context::Cx,
    router::{error::not_found, href, page, path_param, query_params},
    view::{View, view},
};

use super::recipes::{self, recipe_id};
use super::{
    ACTIONS, DONE, H1, H2, LINK, MAIN, TABLE, TD, TH, account, act, act_for_good, done, followed,
    not_done, notice, pager, read, search, token, when,
};
use crate::api::CommandError;
use crate::app::{api, db};
use crate::db::Db;
use crate::listing::{PAGE_SIZE, Pages, searched};
use crate::users::UserRow;

/// How many recipes the page of a user lists.
const RECIPES_SHOWN: i64 = 20;

path_param!(pub user_id);

#[query_params]
struct Query {
    q: Option<String>,
    page: Option<u32>,
}

#[query_params]
struct Shown {
    /// What was just done to the user.
    done: Option<String>,
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
                            <td class=(TD)>(role(user))</td>
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
fn role(user: &UserRow) -> &'static str {
    match (user.deleted, user.is_suspended(), user.is_admin) {
        (true, _, _) => "Deleted",
        (false, true, _) => "Suspended",
        (false, false, true) => "Administrator",
        (false, false, false) => "User",
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
    let suspended_by = match &user.suspended_by {
        Some(by) => read(crate::users::get(db, by).await)?,
        None => None,
    };
    let done = query_params::<Shown>(cx).ok();
    let done = match done.and_then(|shown| shown.done.as_deref()) {
        Some("suspended") => Some(("suspended", "This account is suspended.")),
        Some("reactivated") => Some(("reactivated", "This account is reactivated.")),
        Some("deleted") => Some(("deleted", "This account is deleted.")),
        _ => None,
    };
    // Nobody does any of it to themself or to root, and the api decides
    // for the others.
    let acts = !user.deleted && user.id != account(cx).id && user.id != ROOT_USER_ID;
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
            if let Some((what, message)) = done {
                notice(what: what, message: message)
            }
            <dl class="mt-6 grid gap-x-8 gap-y-3 text-sm sm:grid-cols-2">
                <div><dt class="text-stone-500">"Address"</dt><dd>(user.account.as_str())</dd></div>
                <div>
                    <dt class="text-stone-500">"Role"</dt>
                    <dd data-role="">(role(&user))</dd>
                </div>
                if let Some(since) = user.suspended_at {
                    <div class="sm:col-span-2">
                        <dt class="text-stone-500">"Suspended"</dt>
                        <dd data-suspended="">
                            (when(since))
                            if let Some(by) = &suspended_by {
                                " by "
                                <a class=(LINK) href=(href!(show, user_id: by.id.as_str()))>
                                    (by.label.as_str())
                                </a>
                            }
                        </dd>
                    </div>
                }
                <div><dt class="text-stone-500">"Registered"</dt><dd>(when(user.registered_at))</dd></div>
                <div>
                    <dt class="text-stone-500">"Last sign-in"</dt>
                    <dd>(user.last_sign_in_at.map(when).unwrap_or_else(|| "Never".to_owned()))</dd>
                </div>
                <div><dt class="text-stone-500">"Sign-ins"</dt><dd>(user.sign_in_count)</dd></div>
                <div><dt class="text-stone-500">"Signs in with"</dt><dd>(methods)</dd></div>
                <div><dt class="text-stone-500">"Id"</dt><dd class="break-all">(user.id.as_str())</dd></div>
            </dl>

            if acts {
                <h2 class=(H2)>"Account"</h2>
                <div class=(ACTIONS)>
                    if user.is_suspended() {
                        act(
                            action: href!(reactivate, user_id: user.id.as_str()).resolve(cx),
                            what: "reactivate",
                            label: "Reactivate",
                        )
                    } else {
                        act(
                            action: href!(suspend, user_id: user.id.as_str()).resolve(cx),
                            what: "suspend",
                            label: "Suspend",
                        )
                    }
                    act_for_good(
                        action: href!(delete, user_id: user.id.as_str()).resolve(cx),
                        what: "delete",
                        label: "Delete the account",
                        warning: "The account is gone and its sessions end. The person may register again; what they kept is not given back to them.",
                    )
                </div>
                if !user.is_suspended() {
                    <p class="mt-3 text-sm text-stone-600">
                        "Suspended, they are signed out everywhere and cannot sign in until reactivated. Their account and what they keep stay."
                    </p>
                }
            }

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

/// Whether the user `id` is as `is`, as far as admin has followed: one it
/// does not know is.
async fn user_is(db: &Db, id: &str, is: impl Fn(&UserRow) -> bool) -> anyhow::Result<bool> {
    Ok(crate::users::get(db, id)
        .await?
        .is_none_or(|user| is(&user)))
}

/// Sends the browser back to the user `id` once `outcome` is done, to be
/// told `what` was; the page of what was not otherwise.
fn back(cx: &Cx, id: &str, outcome: Result<(), CommandError>, what: &str) -> Result<impl View> {
    let then = href!(show, user_id: id).query([(DONE, what)]).resolve(cx);
    let failure = done(cx, outcome, then)?;
    Ok(view! { not_done(failure: failure, back: href!(show, user_id: id).resolve(cx)) })
}

#[page(POST "./{user_id}/suspend")]
async fn suspend(cx: &Cx) -> Result<impl View> {
    let id = user_id(cx);
    let outcome = api(cx).suspend_user(&token(cx)?, id).await;
    if outcome.is_ok() {
        followed(|| user_is(db(cx), id, UserRow::is_suspended)).await;
    }
    back(cx, id, outcome, "suspended")
}

#[page(POST "./{user_id}/reactivate")]
async fn reactivate(cx: &Cx) -> Result<impl View> {
    let id = user_id(cx);
    let outcome = api(cx).reactivate_user(&token(cx)?, id).await;
    if outcome.is_ok() {
        followed(|| user_is(db(cx), id, |user| !user.is_suspended())).await;
    }
    back(cx, id, outcome, "reactivated")
}

#[page(POST "./{user_id}/delete")]
async fn delete(cx: &Cx) -> Result<impl View> {
    let id = user_id(cx);
    let outcome = api(cx).delete_user(&token(cx)?, id).await;
    if outcome.is_ok() {
        followed(|| user_is(db(cx), id, |user| user.deleted)).await;
    }
    back(cx, id, outcome, "deleted")
}
