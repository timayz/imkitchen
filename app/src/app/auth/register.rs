//! `/auth/register`: where a visitor creates their account, and is
//! signed in with it.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{content::Form, href, page},
    view::{View, component, view},
};

use super::{Doing, Filled, Posted, attempt, form, sign_in, visitors_only};
use crate::app::t;

#[page(with(Metadata::new().title(t!(register_meta_title))))]
pub async fn register(cx: &Cx) -> Result<impl View> {
    visitors_only(cx)?;

    Ok(view! { content(filled: Filled::blank()) })
}

#[page(POST, with(Metadata::new().title(t!(register_meta_title))))]
async fn submit(cx: &Cx, Form(posted): Form<Posted>) -> Result<impl View> {
    let filled = attempt(cx, Doing::Registering, posted).await?;

    Ok(view! { content(filled: filled) })
}

#[component]
async fn content(cx: &Cx, filled: Filled) -> Result<impl View> {
    let action = href!(submit).resolve(cx).to_string();
    let other = href!(sign_in::sign_in).resolve(cx).to_string();

    Ok(view! {
        form(
            doing: Doing::Registering,
            filled: filled,
            title: t!(register_title),
            submit: t!(register_submit),
            action: action,
            other: other,
            other_label: t!(register_to_sign_in),
        )
    })
}
