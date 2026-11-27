//! `/settings`: how the list is liked, and the account. Nothing of them is
//! shown yet: the screen is there for the navigation to lead to.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::app::{_signed_in::screen, t};

#[page(with(Metadata::new().title(t!(settings_meta_title))))]
pub async fn show(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        screen(title: t!(settings_title), text: t!(settings_text))
    })
}
