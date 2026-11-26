//! The pages: `/` is the dashboard, and `/health` is for the probes.
//!
//! admin is read by the people who run imkitchen, in English: its sentences
//! are in the pages. No search engine is to list any of them.

pub mod health;

use topcoat::{
    Error, Result,
    asset::Asset,
    context::{Cx, context},
    metadata::Metadata,
    router::{
        RouterBuilder, Slot, error::NotFoundError, href, layout, module_router, not_found, page,
    },
    tailwind,
    view::{View, error_boundary, view},
};

use crate::db::Db;
use crate::events::Events;
use crate::users;

/// The stylesheet of every page, as `build.rs` compiles it.
pub const STYLESHEET: Asset = tailwind::stylesheet!();

/// The pages of admin, read from `db` and saying how admin stands with the
/// api through `events`. Give the result its assets, then build it.
pub fn router(db: Db, events: Events) -> RouterBuilder {
    module_router!().with(db).with(events)
}

/// The read models of admin.
pub(crate) fn db(cx: &Cx) -> &Db {
    context(cx)
}

// Anything else is the "not found" page.
not_found!();

#[layout(with(Metadata::new().title("imkitchen admin")))]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                topcoat::metadata::tags()
                <meta name="robots" content="noindex, nofollow">
                <link rel="stylesheet" href=(STYLESHEET)>
                topcoat::dev::script()
            </head>
            <body class="min-h-screen bg-stone-50 text-stone-900 antialiased">
                <header class="mx-auto flex max-w-5xl items-center justify-between px-4 py-6">
                    <a class="text-xl font-semibold tracking-tight" href=(href!(dashboard))>
                        "imkitchen admin"
                    </a>
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
            </body>
        </html>
    })
}

#[page]
pub async fn dashboard(cx: &Cx) -> Result<impl View> {
    let users = users::count(db(cx)).await.map_err(Error::from_anyhow)?;

    Ok(view! {
        <main class="mx-auto max-w-5xl px-4 py-10">
            <h1 class="text-3xl font-bold tracking-tight">"Dashboard"</h1>
            <dl class="mt-8 grid gap-4 sm:grid-cols-3">
                <div class="rounded-lg border border-stone-200 bg-white p-6">
                    <dt class="text-sm text-stone-500">"Users"</dt>
                    <dd class="mt-2 text-3xl font-semibold" data-users=(users)>(users)</dd>
                </div>
            </dl>
        </main>
    })
}
