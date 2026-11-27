//! The pages of one language, under its code: `/en`, `/fr`.

pub mod contact;
pub mod privacy;
pub mod terms;

use topcoat::{
    Error, Result,
    context::Cx,
    l10n::{Locale, Translation},
    router::{
        Body, Next, error::not_found, href, layer, module_param, not_found, page, request::uri,
        response::Response,
    },
    view::{View, component, view},
};

use super::{db, locales, site, t};
use crate::recipes;

module_param!(pub language: Locale else not_found);

// Anything else under a language is that language's "not found" page.
not_found!();

/// The pages of a language, as what follows its code in their address: the
/// home, then one per module below. The sitemap lists them, and each names
/// itself in the other languages.
pub(crate) const PAGES: [&str; 4] = ["", "/terms", "/privacy", "/contact"];

/// The address of `page`, one of [`PAGES`], in `language`.
pub(crate) fn address(language: &Locale, page: &str) -> String {
    format!("/{}{page}", language.id())
}

/// Which of [`PAGES`] is at `path` in `language`, when one is.
pub(crate) fn page(path: &str, language: &Locale) -> Option<&'static str> {
    let rest = path.strip_prefix(&address(language, ""))?;
    PAGES.into_iter().find(|page| *page == rest)
}

/// The pages below are read in the language of their address. A code www
/// has no sentences for is the address of nothing, and neither is another
/// way to write one it has (`/EN`): a page has one address.
#[layer]
async fn spoken(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let language = language(cx)?;
    let written = uri(cx)
        .path()
        .strip_prefix(&address(language, ""))
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'));
    if !written || !locales().contains(language) {
        return Err(not_found().into());
    }
    next.run(&cx.with(language.clone()), body).await
}

#[page]
pub async fn home(cx: &Cx) -> Result<impl View> {
    let recipes = recipes::count(db(cx)).await.map_err(Error::from_anyhow)?;
    let features = [
        (t!(home_recipes_title), t!(home_recipes_text)),
        (t!(home_dishes_title), t!(home_dishes_text)),
        (t!(home_list_title), t!(home_list_text)),
        (t!(home_groceries_title), t!(home_groceries_text)),
    ];
    let app = &site(cx).app;
    // The listings that are live, each under the sentence of its link.
    let listings: Vec<_> = [
        (t!(get_web), &app.web),
        (t!(get_google_play), &app.google_play),
        (t!(get_app_store), &app.app_store),
        (t!(get_desktop), &app.desktop),
    ]
    .into_iter()
    .filter_map(|(label, link)| Some((label, link.as_deref()?)))
    .collect();

    Ok(view! {
        <main>
            <section class="mx-auto max-w-5xl px-4 pt-12 pb-16 sm:pt-20">
                <h1 class="max-w-2xl text-4xl font-bold tracking-tight sm:text-6xl">
                    (t!(home_title))
                </h1>
                <p class="mt-6 max-w-2xl text-lg leading-relaxed text-stone-600">
                    (t!(home_text))
                </p>
                <a
                    class="mt-8 inline-block rounded-lg bg-stone-900 px-5 py-3 font-semibold text-stone-50 hover:bg-stone-700"
                    href="#get"
                >
                    (t!(nav_get))
                </a>
            </section>
            <section
                class="border-y border-stone-200 bg-white"
                aria-labelledby="features"
            >
                <div class="mx-auto max-w-5xl px-4 py-16">
                    <h2 id="features" class="text-2xl font-bold tracking-tight sm:text-3xl">
                        (t!(home_features_title))
                    </h2>
                    <ul class="mt-8 grid gap-8 sm:grid-cols-2">
                        for (title, text) in features {
                            <li>
                                <h3 class="text-lg font-semibold">(title)</h3>
                                <p class="mt-2 leading-relaxed text-stone-600">(text)</p>
                            </li>
                        }
                    </ul>
                </div>
            </section>
            <section class="mx-auto max-w-5xl px-4 py-16" aria-labelledby="start">
                <h2 id="start" class="text-2xl font-bold tracking-tight sm:text-3xl">
                    (t!(home_start_title))
                </h2>
                <p class="mt-4 max-w-2xl leading-relaxed text-stone-600">
                    (t!(home_start_text))
                </p>
            </section>
            <section
                id="get"
                class="border-t border-stone-200 bg-white"
                aria-labelledby="get-title"
            >
                <div class="mx-auto max-w-5xl px-4 py-16">
                    <h2 id="get-title" class="text-2xl font-bold tracking-tight sm:text-3xl">
                        (t!(get_title))
                    </h2>
                    <p class="mt-4 max-w-2xl leading-relaxed text-stone-600">(t!(get_text))</p>
                    if listings.is_empty() {
                        <p class="mt-4 max-w-2xl leading-relaxed text-stone-600" data-get="coming">
                            (t!(get_coming))
                        </p>
                    } else {
                        <ul class="mt-8 flex flex-wrap gap-3">
                            for (label, link) in listings {
                                <li>
                                    <a
                                        class="inline-block rounded-lg bg-stone-900 px-5 py-3 font-semibold text-stone-50 hover:bg-stone-700"
                                        href=(link)
                                        rel="noopener"
                                    >
                                        (label)
                                    </a>
                                </li>
                            }
                        </ul>
                    }
                    <p class="mt-8 text-sm text-stone-500" data-recipes=(recipes)>
                        (t!(home_recipes, count: recipes))
                    </p>
                </div>
            </section>
        </main>
    })
}

/// A part of a legal page: its title, its paragraphs, then the facts it
/// lists, each under its name.
pub(crate) struct Section {
    pub title: Translation<'static>,
    pub paragraphs: Vec<Translation<'static>>,
    pub facts: Vec<(Translation<'static>, String)>,
}

impl Section {
    pub fn new<const N: usize>(
        title: Translation<'static>,
        paragraphs: [Translation<'static>; N],
    ) -> Self {
        Self {
            title,
            paragraphs: paragraphs.into(),
            facts: Vec::new(),
        }
    }

    pub fn facts<const N: usize>(mut self, facts: [(Translation<'static>, &str); N]) -> Self {
        self.facts = facts
            .into_iter()
            .map(|(name, fact)| (name, fact.to_owned()))
            .collect();
        self
    }
}

/// A legal page: what it is, in short, then each of its parts.
#[component]
pub(crate) async fn document(
    cx: &Cx,
    title: Translation<'static>,
    intro: Translation<'static>,
    sections: Vec<Section>,
) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 pt-12 pb-16">
            <h1 class="text-4xl font-bold tracking-tight">(title)</h1>
            <p class="mt-6 text-lg leading-relaxed text-stone-600">(intro)</p>
            <p class="mt-2 text-sm text-stone-500">(t!(legal_updated))</p>
            for section in sections {
                <section class="mt-10">
                    <h2 class="text-xl font-semibold">(section.title)</h2>
                    for paragraph in section.paragraphs {
                        <p class="mt-3 leading-relaxed text-stone-700">(paragraph)</p>
                    }
                    if !section.facts.is_empty() {
                        <dl class="mt-4 grid gap-x-6 gap-y-2 text-stone-700 sm:grid-cols-[12rem_1fr]">
                            for (name, fact) in section.facts {
                                <dt class="font-medium text-stone-900">(name)</dt>
                                <dd>(fact)</dd>
                            }
                        </dl>
                    }
                </section>
            }
            <p class="mt-10 text-sm text-stone-500">
                <a class="underline" href=(href!(contact::contact, language: language(cx)?))>
                    (t!(legal_contact))
                </a>
            </p>
        </main>
    })
}
