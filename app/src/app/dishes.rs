//! `/dishes`: the dishes composed from the recipes. Nothing of them is shown
//! yet: the screen is there for the navigation to lead to.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::app::{screen, t};

#[page(with(Metadata::new().title(t!(dishes_meta_title))))]
pub async fn list(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        screen(title: t!(dishes_title), text: t!(dishes_text))
    })
}
