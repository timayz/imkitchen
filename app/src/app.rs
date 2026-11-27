//! The pages: `/` is the home, each screen of the application (the recipes,
//! the dishes, the list, the groceries, the settings) is at the address of
//! its module, and `/health` is for the probes.
//!
//! Every page is read from the store of app, which its subscriptions fill
//! from the api's events: none asks the api for what it shows.
//!
//! One layout serves the desktop and the phone: the navigation between the
//! screens is a column beside the page where there is room for one, and a
//! bar under it where there is not.
//!
//! A page is read in the language the browser prefers among those app
//! speaks: its address is the same in each. The sentences are in
//! `app/l10n`, one file per language: a language is spoken once its file is
//! there. No search engine is to list any page.

pub mod dishes;
pub mod groceries;
pub mod health;
pub mod list;
pub mod recipes;
pub mod settings;

use topcoat::{
    Error, Result,
    asset::Asset,
    context::{Cx, context},
    l10n::{Locale, Translation, accept_language, locale, negotiate},
    metadata::{Metadata, Robots},
    router::{
        Body, Next, RouterBuilder, Slot, StatusCode, error::NotFoundError, href, layer, layout,
        module_router, not_found, page, request::uri, response::Response,
    },
    tailwind,
    view::{View, class, component, error_boundary, view},
};

use crate::db::Db;
use crate::events::Events;

topcoat::l10n::fluent::messages!(pub(crate) "{mod}/l10n/{locale}.ftl");

/// The stylesheet of every page, as `build.rs` compiles it.
pub const STYLESHEET: Asset = tailwind::stylesheet!();

/// The language of a reader whose browser asks for none app speaks.
const DEFAULT_LOCALE: Locale = locale!("en");

/// The pages of app, read from `db`, saying how app stands with the api
/// through `events`. Give the result its assets, then build it.
pub fn router(db: Db, events: Events) -> RouterBuilder {
    module_router!().with(db).with(events)
}

/// The read models of app.
pub(crate) fn db(cx: &Cx) -> &Db {
    context(cx)
}

// Anything else is the "not found" page.
not_found!();

/// Every page is read in the language the browser prefers among those app
/// speaks.
#[layer]
async fn preferred_language(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let preferred = negotiate(accept_language(cx), t::locales())
        .cloned()
        .unwrap_or(DEFAULT_LOCALE);
    next.run(&cx.with(preferred), body).await
}

/// A screen of the application, as the navigation leads to it.
struct Screen {
    address: String,
    name: Translation<'static>,
    /// Whether it is what is being read.
    current: bool,
}

/// The screens, in the order the navigation gives them.
fn screens(cx: &Cx) -> Vec<Screen> {
    let reading = uri(cx).path();
    [
        (href!(recipes::list).resolve(cx), t!(recipes_title)),
        (href!(dishes::list).resolve(cx), t!(dishes_title)),
        (href!(list::show).resolve(cx), t!(list_title)),
        (href!(groceries::show).resolve(cx), t!(groceries_title)),
        (href!(settings::show).resolve(cx), t!(settings_title)),
    ]
    .into_iter()
    .map(|(address, name)| {
        let address = address.to_string();
        Screen {
            // A screen is current on the pages under it too.
            current: reading
                .strip_prefix(address.as_str())
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('/')),
            address,
            name,
        }
    })
    .collect()
}

#[layout(with(
    Metadata::new()
        .title(t!(site_title))
        .robots(Robots::new().index(false).follow(false))
))]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let current = locale(cx);
    let screens = screens(cx);
    let brand = href!(home).resolve(cx).to_string();
    let back = brand.clone();

    Ok(view! {
        <!DOCTYPE html>
        <html lang=(current.id()) dir=(current.id().direction())>
            <head>
                topcoat::metadata::tags()
                <link rel="stylesheet" href=(STYLESHEET)>
                topcoat::dev::script()
            </head>
            // A phone: the name on top, the page, the screens in a bar that
            // stays at the bottom. From `md` up: the name and the screens in
            // a column that stays beside the page.
            <body class="min-h-dvh bg-stone-50 text-stone-900 antialiased md:flex">
                <header
                    class="border-b border-stone-200 bg-white md:sticky md:top-0 md:flex md:h-dvh md:w-56 md:shrink-0 md:flex-col md:border-r md:border-b-0"
                >
                    <a
                        class="block px-4 py-3 text-xl font-semibold tracking-tight md:px-5 md:py-6"
                        href=(brand)
                        aria-label=(t!(nav_home))
                    >
                        "imkitchen"
                    </a>
                    <nav
                        class="fixed inset-x-0 bottom-0 z-10 flex border-t border-stone-200 bg-white md:static md:flex-col md:gap-1 md:border-t-0 md:px-3"
                        aria-label=(t!(nav_screens))
                    >
                        for screen in screens {
                            <a
                                class=(class!(
                                    "min-w-0 flex-1 truncate px-1 py-3 text-center text-xs md:flex-none md:rounded-lg md:px-3 md:py-2 md:text-left md:text-sm",
                                    "font-semibold text-stone-900 md:bg-stone-100" if screen.current,
                                    "text-stone-500 hover:text-stone-900" if !screen.current,
                                ))
                                href=(screen.address)
                                aria-current=(screen.current.then_some("page"))
                            >
                                (screen.name)
                            </a>
                        }
                    </nav>
                </header>
                // The bar of a phone covers the bottom of the page: the page
                // ends above it.
                <div class="min-w-0 flex-1 pb-16 md:pb-0">
                    error_boundary(
                        fallback: |error| {
                            if error.downcast_ref::<NotFoundError>().is_none() {
                                return Err(error);
                            }

                            Ok(
                                view! {
                                    (StatusCode::NOT_FOUND)
                                    <main class="mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12">
                                        <h1 class="text-3xl font-bold tracking-tight">
                                            (t!(not_found_title))
                                        </h1>
                                        <p class="mt-4 text-stone-600">(t!(not_found_text))</p>
                                        <p class="mt-6">
                                            <a class="underline" href=(back)>
                                                (t!(not_found_home))
                                            </a>
                                        </p>
                                    </main>
                                },
                            )
                        },
                        (slot)
                    )
                </div>
            </body>
        </html>
    })
}

/// The home: what the application is, and how many recipes app has
/// followed. Whose they are is for the pages to know once someone signs in.
#[page]
pub async fn home(cx: &Cx) -> Result<impl View> {
    let recipes = crate::recipes::count(db(cx))
        .await
        .map_err(Error::from_anyhow)?;

    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12">
            <h1 class="text-3xl font-bold tracking-tight md:text-4xl">(t!(home_title))</h1>
            <p class="mt-4 text-lg leading-relaxed text-stone-600">(t!(home_text))</p>
            <p class="mt-8 text-sm text-stone-500" data-recipes=(recipes)>
                (t!(home_recipes, count: recipes))
            </p>
        </main>
    })
}

/// A screen nothing is shown on yet: what it is, and what it will hold.
#[component]
pub(crate) async fn screen(
    cx: &Cx,
    title: Translation<'static>,
    text: Translation<'static>,
) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12">
            <h1 class="text-3xl font-bold tracking-tight md:text-4xl">(title)</h1>
            <p class="mt-4 text-lg leading-relaxed text-stone-600">(text)</p>
            <p class="mt-8 text-sm text-stone-500" data-screen="coming">(t!(screen_coming))</p>
        </main>
    })
}
