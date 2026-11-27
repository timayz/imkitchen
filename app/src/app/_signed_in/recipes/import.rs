//! `/recipes/import`: where a recipe is imported from a page of the web.
//!
//! The address of the page is all that is asked: the api fetches it and
//! reads the recipe it holds. What it gives is a recipe of who asked, theirs
//! to rewrite like one they wrote, and the browser is sent to it. A page
//! the api cannot read, or reads no recipe on, is said as the api words it.

use imkitchen_aggregates::tutorial::Step;
use serde::{Deserialize, Serialize};
use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{content::Form, error::see_other, href, page},
    view::{View, component, view},
};

use super::{Told, caught_up, list, recipe_id, show};
use crate::app::_signed_in::{
    BUTTON, FIELD_REFUSED, Failure, H1, INPUT, LABEL, LINK, MAIN, asking, failed, tutorial,
};
use crate::app::{api, language, t};

/// What the form posts, and what the api is sent.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct Posted {
    url: String,
}

#[page(with(Metadata::new().title(t!(import_meta_title))))]
pub async fn ask(cx: &Cx) -> Result<impl View> {
    Ok(view! { content(url: String::new(), failure: None) })
}

#[page(POST, with(Metadata::new().title(t!(import_meta_title))))]
async fn submit(cx: &Cx, Form(posted): Form<Posted>) -> Result<impl View> {
    let language = language(cx);
    let imported = api(cx)
        .create::<_, Told>(asking(cx, &language), &["recipes", "import"], &posted)
        .await;
    let failure = match imported {
        Ok(told) => {
            // A recipe imported is the first step of the tutorial too.
            tokio::join!(caught_up(cx, &told), tutorial::noted(cx, Step::Recipe));
            let then = href!(show, recipe_id: told.id.as_str()).resolve(cx);
            return Err(see_other(then).into());
        }
        Err(err) => Failure::of(cx, err)?,
    };

    Ok(view! { content(url: posted.url, failure: Some(failure)) })
}

#[component]
async fn content(cx: &Cx, url: String, failure: Option<Failure>) -> Result<impl View> {
    let url_refused = failure.as_ref().and_then(|failure| failure.field("url"));

    Ok(view! {
        <main class=(MAIN)>
            <p class="text-sm"><a class=(LINK) href=(href!(list))>(t!(recipes_title))</a></p>
            <h1 class=(H1)>(t!(import_title))</h1>
            <p class="mt-4 text-lg leading-relaxed text-stone-600">(t!(import_text))</p>
            <form class="mt-8 grid gap-5" method="post" action=(href!(submit))>
                failed(failure: failure)
                <div>
                    <label class=(LABEL) for="import-url">(t!(import_url))</label>
                    <input
                        id="import-url"
                        class=(INPUT)
                        type="url"
                        name="url"
                        value=(url)
                        placeholder="https://"
                        aria-invalid=(url_refused.is_some().then_some("true"))
                        required=""
                    >
                    if let Some(message) = url_refused {
                        <p class=(FIELD_REFUSED) data-field="url">(message)</p>
                    }
                </div>
                <div>
                    <button class=(BUTTON) type="submit">(t!(import_submit))</button>
                </div>
            </form>
        </main>
    })
}
