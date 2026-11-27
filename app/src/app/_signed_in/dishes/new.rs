//! `/dishes/new`: where a dish is composed, with the form one is recomposed
//! with ([`super::_form`]).

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{content::Form, page},
    view::{View, view},
};

use super::_form::{Again, Typed, form, hand};
use crate::app::t;

#[page(with(Metadata::new().title(t!(dish_new_meta_title))))]
pub async fn blank(cx: &Cx) -> Result<impl View> {
    Ok(view! { form(again: Again::of(Typed::default()), recomposes: None) })
}

#[page(POST, with(Metadata::new().title(t!(dish_new_meta_title))))]
pub(crate) async fn submit(
    cx: &Cx,
    Form(posted): Form<Vec<(String, String)>>,
) -> Result<impl View> {
    let again = hand(cx, None, posted).await?;

    Ok(view! { form(again: again, recomposes: None) })
}
