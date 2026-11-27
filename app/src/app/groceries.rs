//! `/groceries`: the groceries of the list. Nothing of them is shown yet: the
//! screen is there for the navigation to lead to.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::app::{screen, t};

#[page(with(Metadata::new().title(t!(groceries_meta_title))))]
pub async fn show(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        screen(title: t!(groceries_title), text: t!(groceries_text))
    })
}
