//! The pages: `/` sends a visitor to the home of their language, `/en` and
//! `/fr` are that home, and `/health` is for the probes.
//!
//! Every page is under its language, the first part of its address
//! ([`language`]), so each language has addresses of its own for a search
//! engine to list. The sentences are in `app/l10n`, one file per language:
//! a language is spoken once its file is there.

pub mod health;
pub mod language;

use topcoat::{
    Result,
    asset::Asset,
    context::{Cx, context},
    l10n::{Locale, accept_language, locale, negotiate},
    metadata::Metadata,
    router::{
        Body, Next, RouterBuilder, Slot,
        error::{NotFoundError, redirect},
        href, layer, layout, module_router,
        response::Response,
        route,
    },
    tailwind,
    view::{View, class, error_boundary, view},
};

use crate::db::Db;
use crate::events::Events;

topcoat::l10n::fluent::messages!(pub(crate) "{mod}/l10n/{locale}.ftl");

/// The stylesheet of every page, as `build.rs` compiles it.
pub const STYLESHEET: Asset = tailwind::stylesheet!();

/// The language of a visitor whose browser asks for none www speaks.
const DEFAULT_LOCALE: Locale = locale!("en");

/// The pages of www, read from `db` and saying how www stands with the api
/// through `events`. Give the result its assets, then build it.
pub fn router(db: Db, events: Events) -> RouterBuilder {
    module_router!().with(db).with(events)
}

/// The read models of www.
pub(crate) fn db(cx: &Cx) -> &Db {
    context(cx)
}

/// The languages www speaks: those with a file in `app/l10n`.
pub(crate) fn locales() -> &'static [Locale] {
    t::locales()
}

/// Before a page says which language it is in, the one the browser prefers
/// among those www speaks.
#[layer]
async fn preferred_language(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let preferred = negotiate(accept_language(cx), locales())
        .cloned()
        .unwrap_or(DEFAULT_LOCALE);
    next.run(&cx.with(preferred), body).await
}

/// No page is at `/`: the home of the visitor's language is.
#[route(GET)]
async fn index(cx: &Cx) -> Result<()> {
    let home = href!(language::home, language: locale(cx));
    Err(redirect(home.resolve(cx)).into())
}

#[layout(with(Metadata::new().title(t!(site_title)).description(t!(site_description))))]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let current = locale(cx);

    Ok(view! {
        <!DOCTYPE html>
        <html lang=(current.id()) dir=(current.id().direction())>
            <head>
                topcoat::metadata::tags()
                <link rel="stylesheet" href=(STYLESHEET)>
                topcoat::dev::script()
            </head>
            <body class="min-h-screen bg-stone-50 text-stone-900 antialiased">
                <header
                    class="mx-auto flex max-w-3xl items-center justify-between px-4 py-6"
                >
                    <a
                        class="text-xl font-semibold tracking-tight"
                        href=(href!(language::home, language: current))
                    >
                        "imkitchen"
                    </a>
                    <nav class="flex gap-4 text-sm" aria-label=(t!(languages))>
                        for spoken in locales() {
                            <a
                                class=(class!("hover:underline", "font-semibold" if spoken == current))
                                href=(href!(language::home, language: spoken))
                                hreflang=(spoken.id())
                                lang=(spoken.id())
                                aria-current=((spoken == current).then_some("page"))
                            >
                                (t!(language_name).resolve_in(spoken))
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
                                <main class="mx-auto max-w-3xl px-4 py-16">
                                    <h1 class="text-3xl font-bold">(t!(not_found_title))</h1>
                                    <p class="mt-4 text-stone-600">(t!(not_found_text))</p>
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
