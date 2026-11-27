//! `/recipes`: the recipes of who reads the page. Nothing of them is shown
//! yet: the screen is there for the navigation to lead to.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::app::{screen, t};

#[page(with(Metadata::new().title(t!(recipes_meta_title))))]
pub async fn list(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        screen(title: t!(recipes_title), text: t!(recipes_text))
    })
}
