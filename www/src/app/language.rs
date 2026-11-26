//! The pages of one language, under its code: `/en`, `/fr`.

use topcoat::{
    Error, Result,
    context::Cx,
    l10n::Locale,
    router::{
        Body, Next, error::not_found, layer, module_param, not_found, page, response::Response,
    },
    view::{View, view},
};

use super::{db, locales, t};
use crate::recipes;

module_param!(pub language: Locale else not_found);

// Anything else under a language is that language's "not found" page.
not_found!();

/// The pages below are read in the language of their address. A code www
/// has no sentences for is the address of nothing.
#[layer]
async fn spoken(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let language = language(cx)?;
    if !locales().contains(language) {
        return Err(not_found().into());
    }
    next.run(&cx.with(language.clone()), body).await
}

#[page]
pub async fn home(cx: &Cx) -> Result<impl View> {
    let recipes = recipes::count(db(cx)).await.map_err(Error::from_anyhow)?;

    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 py-16">
            <h1 class="text-4xl font-bold tracking-tight">(t!(home_title))</h1>
            <p class="mt-4 text-lg text-stone-600">(t!(home_text))</p>
            <p class="mt-8 text-sm text-stone-500" data-recipes=(recipes)>
                (t!(home_recipes, count: recipes))
            </p>
        </main>
    })
}
