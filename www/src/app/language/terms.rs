//! `/{language}/terms`: the terms imkitchen is used under, and who
//! publishes it.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use super::{Section, document};
use crate::app::{site, t};

/// Who publishes imkitchen and who hosts it, as the law asks a site to say.
const PUBLISHER: &str = "Timada";
const PUBLICATION_DIRECTOR: &str = "Jonathan LAPIQUONNE";
const PUBLISHER_ADDRESS: &str = "60 rue François Ier, c/o LegalPlace, 75008 Paris, France";
const PUBLISHER_SIRET: &str = "989 261 383 00016";
const HOST: &str = "OVHcloud, 2 rue Kellermann, 59100 Roubaix, France";

#[page(with(Metadata::new().title(t!(terms_meta_title)).description(t!(terms_description))))]
pub async fn terms(cx: &Cx) -> Result<impl View> {
    let sections = vec![
        Section::new(
            t!(terms_agreement_title),
            [t!(terms_agreement_1), t!(terms_agreement_2)],
        ),
        Section::new(
            t!(terms_account_title),
            [t!(terms_account_1), t!(terms_account_2)],
        ),
        Section::new(
            t!(terms_content_title),
            [t!(terms_content_1), t!(terms_content_2)],
        ),
        Section::new(
            t!(terms_import_title),
            [t!(terms_import_1), t!(terms_import_2)],
        ),
        Section::new(t!(terms_use_title), [t!(terms_use_1), t!(terms_use_2)]),
        Section::new(t!(terms_property_title), [t!(terms_property_1)]),
        Section::new(
            t!(terms_warranty_title),
            [t!(terms_warranty_1), t!(terms_warranty_2)],
        ),
        Section::new(t!(terms_liability_title), [t!(terms_liability_1)]),
        Section::new(t!(terms_closing_title), [t!(terms_closing_1)]),
        Section::new(t!(terms_changes_title), [t!(terms_changes_1)]),
        Section::new(t!(terms_law_title), [t!(terms_law_1)]),
        Section::new(t!(terms_publisher_title), [t!(terms_publisher_1)]).facts([
            (t!(terms_publisher_name), PUBLISHER),
            (t!(terms_publisher_director), PUBLICATION_DIRECTOR),
            (t!(terms_publisher_address), PUBLISHER_ADDRESS),
            (t!(terms_publisher_siret), PUBLISHER_SIRET),
            (t!(terms_publisher_email), &site(cx).contact.email),
            (t!(terms_publisher_host), HOST),
        ]),
    ];

    Ok(view! {
        document(title: t!(terms_title), intro: t!(terms_intro), sections: sections)
    })
}
