//! What an administrator reads: every page of admin but the one that signs
//! in. `/` is the dashboard; the users, the recipes, their imports, their
//! pictures and the messages written to imkitchen each have a module below.
//!
//! Every page is read from the store of admin, which its subscriptions
//! fill from the api's events: none asks the api for what it shows.
//!
//! The module adds nothing to the addresses of its pages (its name starts
//! with `_`). What it adds is who may read them: whatever is declared here
//! or below is behind [`administrators_only`].

pub mod images;
pub mod imports;
pub mod messages;
pub mod recipes;
pub mod users;

use timada_core::civil::format_time;
use topcoat::{
    Error, Result,
    context::{Cx, context},
    router::{
        Body, Next, Slot,
        error::{NotFoundError, see_other, service_unavailable},
        href, layer, layout, not_found, page,
        request::uri,
        response::Response,
    },
    view::{View, class, component, error_boundary, view},
};

use super::{api, db, sign_in, sign_out};
use crate::api::Account;
use crate::listing::Pages;
use crate::session::{self, Kept};

/// How long a browser waits before it asks again, when the api could not
/// say who it is: in seconds.
const RETRY_AFTER_SECS: u64 = 5;

// Anything else is the "not found" page, of an administrator's too.
not_found!();

/// Who reads the page.
pub(crate) fn account(cx: &Cx) -> &Account {
    context(cx)
}

/// Lets an administrator through, and nobody else: the api is asked whose
/// session the browser keeps on every request, so one that ended, or whose
/// user is an administrator no more, reads nothing further.
#[layer]
async fn administrators_only(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let sign_in = href!(sign_in::sign_in);
    let Some(kept) = session::kept(cx) else {
        return Err(see_other(sign_in.resolve(cx)).into());
    };
    let me = match api(cx).account(&kept.token).await {
        Ok(Some(me)) => me,
        Ok(None) => {
            session::forget(cx);
            return Err(see_other(sign_in.resolve(cx)).into());
        }
        Err(err) => {
            tracing::warn!(
                err = format!("{err:#}"),
                "the api does not say who reads admin"
            );
            return Err(service_unavailable(RETRY_AFTER_SECS).into());
        }
    };
    if !me.user.is_admin() {
        // Its session was opened for admin, which has no use for it.
        if let Err(err) = api(cx).sign_out(&kept.token).await {
            tracing::warn!(err = format!("{err:#}"), "a session was not ended");
        }
        session::forget(cx);
        let refused = sign_in.query([(sign_in::REFUSED, "1")]);
        return Err(see_other(refused.resolve(cx)).into());
    }
    // The api extends a session that is used: so does the browser.
    if me.expires_at > kept.expires_at {
        session::keep(
            cx,
            Kept {
                expires_at: me.expires_at,
                ..kept
            },
        )?;
    }
    next.run(&cx.with(me.user), body).await
}

/// A read of the store that failed, as the error of a page.
pub(crate) fn read<T>(read: anyhow::Result<T>) -> Result<T> {
    read.map_err(Error::from_anyhow)
}

/// A moment (unix seconds) as the pages write it.
pub(crate) fn when(at: i64) -> String {
    format_time(at.max(0) as u64)
}

/// The classes of the tables of the lists, and of their cells.
pub(crate) const TABLE: &str = "mt-6 w-full border-collapse text-left text-sm";
pub(crate) const TH: &str = "border-b border-stone-300 px-3 py-2 font-medium text-stone-500";
pub(crate) const TD: &str = "border-b border-stone-200 px-3 py-2 align-top";
pub(crate) const LINK: &str = "font-medium underline hover:text-stone-600";
pub(crate) const MAIN: &str = "mx-auto max-w-5xl px-4 py-10";
pub(crate) const H1: &str = "text-3xl font-bold tracking-tight";
pub(crate) const H2: &str = "mt-10 text-xl font-semibold tracking-tight";

/// Where a list stands, with the pages before and after when there are
/// some: `previous` and `next` are their addresses.
#[component]
pub(crate) async fn pager(
    cx: &Cx,
    pages: Pages,
    previous: Option<String>,
    next: Option<String>,
) -> Result<impl View> {
    Ok(view! {
        <nav
            class="mt-6 flex items-center justify-between text-sm text-stone-600"
            aria-label="Pages"
            data-total=(pages.total)
        >
            <p>(pages.total) " in all · page " (pages.page) " of " (pages.pages)</p>
            <p class="flex gap-4">
                if let Some(previous) = previous {
                    <a class=(LINK) href=(previous) rel="prev">"Previous"</a>
                }
                if let Some(next) = next {
                    <a class=(LINK) href=(next) rel="next">"Next"</a>
                }
            </p>
        </nav>
    })
}

/// The form a list is searched with: `q` is what was searched for.
#[component]
pub(crate) async fn search(
    cx: &Cx,
    action: String,
    q: String,
    label: &'static str,
) -> Result<impl View> {
    Ok(view! {
        <form class="mt-6 flex gap-2" method="get" action=(action) role="search">
            <input
                class="block w-full max-w-sm rounded-lg border border-stone-300 bg-white px-3 py-2"
                type="search"
                name="q"
                value=(q)
                placeholder=(label)
                aria-label=(label)
            >
            <button
                class="rounded-lg bg-stone-900 px-4 py-2 font-semibold text-stone-50 hover:bg-stone-700"
                type="submit"
            >
                "Search"
            </button>
        </form>
    })
}

#[layout]
async fn signed_in(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let account = account(cx);
    let path = uri(cx).path();
    // The section a page is in is the one its address starts with.
    let current = |address: &str| {
        if address == "/" {
            path == "/"
        } else {
            path == address || path.starts_with(&format!("{address}/"))
        }
    };
    let sections = [
        (href!(dashboard).resolve(cx), "Dashboard"),
        (href!(users::list).resolve(cx), "Users"),
        (href!(recipes::list).resolve(cx), "Recipes"),
        (href!(imports::list).resolve(cx), "Imports"),
        (href!(images::list).resolve(cx), "Pictures"),
        (href!(messages::list).resolve(cx), "Messages"),
    ]
    .map(|(address, name)| (current(&address), address, name));

    Ok(view! {
        <header class="border-b border-stone-200 bg-white">
            <div class="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-x-6 gap-y-2 px-4 py-4">
                <a class="text-xl font-semibold tracking-tight" href=(href!(dashboard))>
                    "imkitchen admin"
                </a>
                <div class="flex items-center gap-4 text-sm">
                    <span class="text-stone-600" data-account=(account.email.as_str())>
                        (account.email.as_str())
                    </span>
                    <form method="post" action=(href!(sign_out))>
                        <button class=(LINK) type="submit">"Sign out"</button>
                    </form>
                </div>
            </div>
            <nav class="mx-auto flex max-w-5xl flex-wrap gap-x-6 gap-y-1 px-4 pb-3 text-sm" aria-label="Sections">
                for (current, address, name) in sections {
                    <a
                        class=(class!("hover:underline", "font-semibold" if current, "text-stone-600" if !current))
                        aria-current=(current.then_some("page"))
                        href=(address)
                    >
                        (name)
                    </a>
                }
            </nav>
        </header>
        error_boundary(
            fallback: |error| {
                if error.downcast_ref::<NotFoundError>().is_none() {
                    return Err(error);
                }

                Ok(
                    view! {
                        (topcoat::router::StatusCode::NOT_FOUND)
                        <main class="mx-auto max-w-5xl px-4 py-16">
                            <h1 class="text-3xl font-bold">"This page does not exist"</h1>
                            <p class="mt-4 text-stone-600">
                                "The address may have changed, or never was one."
                            </p>
                        </main>
                    },
                )
            },
            (slot)
        )
    })
}

/// The key numbers of imkitchen, as far as admin has followed: who uses
/// it and what they keep, and nothing of who visits.
#[page]
pub async fn dashboard(cx: &Cx) -> Result<impl View> {
    let db = db(cx);
    let all = crate::imports::Filter::default();
    let failed = crate::imports::Filter {
        failed: true,
        ..all
    };
    let numbers = [
        (
            "users",
            "Users",
            read(crate::users::count(db).await)?,
            href!(users::list).resolve(cx),
        ),
        (
            "administrators",
            "Administrators",
            read(crate::users::administrators(db).await)?,
            href!(users::list).resolve(cx),
        ),
        (
            "recipes",
            "Recipes",
            read(crate::recipes::count(db, None).await)?,
            href!(recipes::list).resolve(cx),
        ),
        (
            "imports",
            "Imports",
            read(crate::imports::count(db, all).await)?,
            href!(imports::list).resolve(cx),
        ),
        (
            "failed-imports",
            "Failed imports",
            read(crate::imports::count(db, failed).await)?,
            href!(imports::list).query([("failed", "1")]).resolve(cx),
        ),
        (
            "sources",
            "Sources",
            read(crate::imports::count_sources(db).await)?,
            href!(imports::sources).resolve(cx),
        ),
        (
            "pictures",
            "Pictures",
            read(crate::images::count(db, false).await)?,
            href!(images::list).resolve(cx),
        ),
        (
            "shared-pictures",
            "Shared pictures",
            read(crate::images::count(db, true).await)?,
            href!(images::list).query([("shared", "1")]).resolve(cx),
        ),
        (
            "messages",
            "Messages",
            read(crate::messages::count(db).await)?,
            href!(messages::list).resolve(cx),
        ),
    ];

    Ok(view! {
        <main class=(MAIN)>
            <h1 class=(H1)>"Dashboard"</h1>
            <dl class="mt-8 grid gap-4 sm:grid-cols-3">
                for (key, name, number, address) in numbers {
                    <div class="rounded-lg border border-stone-200 bg-white p-6">
                        <dt class="text-sm text-stone-500">
                            <a class="hover:underline" href=(address)>(name)</a>
                        </dt>
                        <dd class="mt-2 text-3xl font-semibold" data-number=(key) data-value=(number)>
                            (number)
                        </dd>
                    </div>
                }
            </dl>
        </main>
    })
}
