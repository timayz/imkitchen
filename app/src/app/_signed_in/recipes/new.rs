//! `/recipes/new`: where a recipe is written by hand, with the form one is
//! rewritten with ([`super::_form`]).

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{content::multipart::Multipart, page},
    view::{View, view},
};

use super::_form::{Again, FORM_LIMIT, Typed, form, hand};
use crate::app::t;

#[page(with(Metadata::new().title(t!(recipe_new_meta_title))))]
pub async fn blank(cx: &Cx) -> Result<impl View> {
    Ok(view! { form(again: Again::of(Typed::empty()), rewrites: None) })
}

#[page(
    POST,
    with(Metadata::new().title(t!(recipe_new_meta_title))),
    with(FORM_LIMIT)
)]
pub(crate) async fn submit(cx: &Cx, multipart: Multipart) -> Result<impl View> {
    let again = hand(cx, None, multipart).await?;

    Ok(view! { form(again: again, rewrites: None) })
}
