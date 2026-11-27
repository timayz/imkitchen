//! `/auth/sign-in`: where a user says who they are.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{content::Form, href, page},
    view::{View, component, view},
};

use super::{Doing, Filled, Posted, attempt, form, register, visitors_only};
use crate::app::t;

#[page(with(Metadata::new().title(t!(sign_in_meta_title))))]
pub async fn sign_in(cx: &Cx) -> Result<impl View> {
    visitors_only(cx)?;

    Ok(view! { content(filled: Filled::blank()) })
}

#[page(POST, with(Metadata::new().title(t!(sign_in_meta_title))))]
async fn submit(cx: &Cx, Form(posted): Form<Posted>) -> Result<impl View> {
    let filled = attempt(cx, Doing::SigningIn, posted).await?;

    Ok(view! { content(filled: filled) })
}

#[component]
async fn content(cx: &Cx, filled: Filled) -> Result<impl View> {
    let action = href!(submit).resolve(cx).to_string();
    let other = href!(register::register).resolve(cx).to_string();

    Ok(view! {
        form(
            doing: Doing::SigningIn,
            filled: filled,
            title: t!(sign_in_title),
            submit: t!(sign_in_submit),
            action: action,
            other: other,
            other_label: t!(sign_in_to_register),
        )
    })
}
