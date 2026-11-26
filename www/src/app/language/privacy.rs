//! `/{language}/privacy`: what imkitchen keeps about its users, and what
//! this site keeps about its visitors, which is nothing.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use super::{Section, document};
use crate::app::{site, t};

#[page(with(Metadata::new().title(t!(privacy_meta_title)).description(t!(privacy_description))))]
pub async fn privacy(cx: &Cx) -> Result<impl View> {
    let email = site(cx).contact.email.clone();
    let sections = vec![
        Section::new(t!(privacy_controller_title), [t!(privacy_controller_1)]),
        Section::new(
            t!(privacy_data_title),
            [
                t!(privacy_data_1),
                t!(privacy_data_2),
                t!(privacy_data_3),
                t!(privacy_data_4),
            ],
        ),
        Section::new(
            t!(privacy_use_title),
            [t!(privacy_use_1), t!(privacy_use_2)],
        ),
        Section::new(t!(privacy_site_title), [t!(privacy_site_1)]),
        Section::new(
            t!(privacy_others_title),
            [t!(privacy_others_1), t!(privacy_others_2)],
        ),
        Section::new(t!(privacy_retention_title), [t!(privacy_retention_1)]),
        Section::new(
            t!(privacy_rights_title),
            [t!(privacy_rights_1, email), t!(privacy_rights_2)],
        ),
        Section::new(t!(privacy_security_title), [t!(privacy_security_1)]),
        Section::new(t!(privacy_children_title), [t!(privacy_children_1)]),
        Section::new(t!(privacy_changes_title), [t!(privacy_changes_1)]),
    ];

    Ok(view! {
        document(title: t!(privacy_title), intro: t!(privacy_intro), sections: sections)
    })
}
