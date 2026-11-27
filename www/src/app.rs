//! The pages: `/` sends a visitor to the home of their language, `/en` and
//! `/fr` are that home, with the legal and contact pages under them, and
//! `/health` is for the probes.
//!
//! Every page is under its language, the first part of its address
//! ([`language`]), so each language has addresses of its own for a search
//! engine to list: `/sitemap.xml` lists them, `/robots.txt` points to it,
//! and each page names its canonical address and itself in the other
//! languages. The sentences are in `app/l10n`, one file per language: a
//! language is spoken once its file is there.
//!
//! No page loads a script or anything from another site: nothing tells
//! anyone a page was read.
//!
//! www answers at two addresses. The main domain, which the gateway in
//! front of imkitchen shares with app: a browser that keeps the marker of a
//! session is served app there, any other www. And `about.`, which is www
//! whoever asks. The pages have one address for a search engine, on the
//! main domain ([`Site::url`]), whichever they are read at, and what www
//! answers is not to be kept for a browser that sends other cookies
//! ([`VariesOnCookies`]).

pub mod health;
pub mod language;

use topcoat::{
    Result,
    asset::Asset,
    context::{BaseUrl, Cx, base_url, context},
    l10n::{Locale, accept_language, locale, negotiate},
    metadata::{Metadata, OpenGraph, Url},
    router::{
        Body, Layer, LayerFuture, Next, Path, RouterBuilder, Slot,
        content::sitemap::{Sitemap, SitemapUrl},
        error::{NotFoundError, redirect},
        header::{HeaderValue, VARY},
        href, layer, layout, module_router,
        request::uri,
        response::{Response, response_headers},
        route,
    },
    tailwind,
    view::{View, class, error_boundary, view},
};

use crate::api::Api;
use crate::config::{AppConfig, ContactConfig};
use crate::db::Db;
use crate::events::Events;

topcoat::l10n::fluent::messages!(pub(crate) "{mod}/l10n/{locale}.ftl");

/// The stylesheet of every page, as `build.rs` compiles it.
pub const STYLESHEET: Asset = tailwind::stylesheet!();

/// The language of a visitor whose browser asks for none www speaks.
const DEFAULT_LOCALE: Locale = locale!("en");

/// What the pages say of imkitchen that a deployment decides.
#[derive(Debug, Clone)]
pub struct Site {
    /// The address visitors reach www at: the main domain, where the pages
    /// have their canonical address.
    pub url: BaseUrl,
    /// Where the application is got.
    pub app: AppConfig,
    /// How a visitor writes to imkitchen.
    pub contact: ContactConfig,
}

/// The pages of www, read from `db`, saying how www stands with the api
/// through `events`, sending their commands to `api` and saying what `site`
/// says of imkitchen. Give the result its assets, then build it.
pub fn router(db: Db, events: Events, api: Api, site: Site) -> RouterBuilder {
    module_router!()
        .layer(VariesOnCookies)
        .with(db)
        .with(events)
        .with(api)
        .with(site.url.clone())
        .with(site)
}

/// The read models of www.
pub(crate) fn db(cx: &Cx) -> &Db {
    context(cx)
}

/// The api, for the commands of the pages.
pub(crate) fn api(cx: &Cx) -> &Api {
    context(cx)
}

/// What a deployment says of imkitchen.
pub(crate) fn site(cx: &Cx) -> &Site {
    context(cx)
}

/// The languages www speaks: those with a file in `app/l10n`.
pub(crate) fn locales() -> &'static [Locale] {
    t::locales()
}

/// On the main domain, what is at an address depends on the cookies the
/// browser sends: www without the marker of a session, app with it. Every
/// answer of www says so, the one of an address of nothing too, for no
/// cache to hand it to a signed-in user.
struct VariesOnCookies;

impl Layer for VariesOnCookies {
    fn path(&self) -> Option<&Path> {
        None
    }

    fn handle<'a>(&'a self, cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
        Box::pin(async move {
            response_headers(cx).append(VARY, HeaderValue::from_static("Cookie"));
            next.run(cx, body).await
        })
    }
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

/// Every page may be listed, and the list is the sitemap.
#[route(GET "./robots.txt")]
async fn robots(cx: &Cx) -> Result<String> {
    Ok(format!(
        "User-agent: *\nAllow: /\n\nSitemap: {}\n",
        href!(sitemap).absolute().resolve(cx)
    ))
}

/// Every page, in every language.
#[route(GET "./sitemap.xml")]
async fn sitemap(cx: &Cx) -> Result<Sitemap> {
    Ok(Sitemap::new().urls(locales().iter().flat_map(|language| {
        language::PAGES.iter().map(move |page| {
            SitemapUrl::new(language::address(language, page)).priority(if page.is_empty() {
                1.0
            } else {
                0.5
            })
        })
    })))
}

#[layout(with(
    Metadata::new()
        .title(t!(site_title))
        .description(t!(site_description))
        .canonical(Url::Current)
        .open_graph(OpenGraph::new().site_name("imkitchen"))
))]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let current = locale(cx);
    // What is being read, when it is a page: each language has it at the
    // same address under its code.
    let page = language::page(uri(cx).path(), current);
    let home = page == Some("");
    let sign_in = site(cx).app.web.clone();

    Ok(view! {
        <!DOCTYPE html>
        <html lang=(current.id()) dir=(current.id().direction())>
            <head>
                topcoat::metadata::tags()
                if let Some(page) = page {
                    for spoken in locales() {
                        <link
                            rel="alternate"
                            hreflang=(spoken.id())
                            href=(base_url(cx).join(&language::address(spoken, page)))
                        >
                    }
                    if home {
                        <link rel="alternate" hreflang="x-default" href=(base_url(cx).join("/"))>
                    }
                }
                <link rel="stylesheet" href=(STYLESHEET)>
                topcoat::dev::script()
            </head>
            <body class="flex min-h-screen flex-col bg-stone-50 text-stone-900 antialiased">
                <header
                    class="mx-auto flex w-full max-w-5xl flex-wrap items-center justify-between gap-x-6 gap-y-2 px-4 py-6"
                >
                    <a
                        class="text-xl font-semibold tracking-tight"
                        href=(href!(language::home, language: current))
                    >
                        "imkitchen"
                    </a>
                    <div class="flex flex-wrap items-center gap-x-6 gap-y-2 text-sm">
                        <nav class="flex gap-4" aria-label=(t!(nav_pages))>
                            <a
                                class="hover:underline"
                                href=(href!(language::home, language: current).fragment("get"))
                            >
                                (t!(nav_get))
                            </a>
                            <a
                                class="hover:underline"
                                href=(href!(language::contact::contact, language: current))
                            >
                                (t!(nav_contact))
                            </a>
                            if let Some(sign_in) = sign_in {
                                <a class="font-semibold hover:underline" href=(sign_in)>
                                    (t!(nav_sign_in))
                                </a>
                            }
                        </nav>
                        <nav class="flex gap-3 text-stone-500" aria-label=(t!(languages))>
                            for spoken in locales() {
                                <a
                                    class=(class!("hover:underline", "font-semibold text-stone-900" if spoken == current))
                                    href=(language::address(spoken, page.unwrap_or_default()))
                                    hreflang=(spoken.id())
                                    lang=(spoken.id())
                                    aria-current=((spoken == current).then_some("page"))
                                >
                                    (t!(language_name).resolve_in(spoken))
                                </a>
                            }
                        </nav>
                    </div>
                </header>
                <div class="flex-1">
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
                </div>
                <footer class="border-t border-stone-200">
                    <div
                        class="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-x-6 gap-y-3 px-4 py-8 text-sm text-stone-600"
                    >
                        <p>"© 2026 imkitchen · " (t!(footer_tagline))</p>
                        <nav class="flex flex-wrap gap-4" aria-label=(t!(footer_pages))>
                            <a
                                class="hover:underline"
                                href=(href!(language::terms::terms, language: current))
                            >
                                (t!(terms_title))
                            </a>
                            <a
                                class="hover:underline"
                                href=(href!(language::privacy::privacy, language: current))
                            >
                                (t!(privacy_title))
                            </a>
                            <a
                                class="hover:underline"
                                href=(href!(language::contact::contact, language: current))
                            >
                                (t!(contact_title))
                            </a>
                        </nav>
                    </div>
                </footer>
            </body>
        </html>
    })
}
