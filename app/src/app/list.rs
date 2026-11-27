//! `/list`: the list of the week. Nothing of it is shown yet: the screen is
//! there for the navigation to lead to.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::app::{screen, t};

#[page(with(Metadata::new().title(t!(list_meta_title))))]
pub async fn show(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        screen(title: t!(list_title), text: t!(list_text))
    })
}
