//! `/{language}/contact`: how a visitor writes to imkitchen.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{href, page},
    view::{View, view},
};

use super::{language, privacy};
use crate::app::{site, t};

#[page(with(Metadata::new().title(t!(contact_meta_title)).description(t!(contact_description))))]
pub async fn contact(cx: &Cx) -> Result<impl View> {
    let email = &site(cx).contact.email;

    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 pt-12 pb-16">
            <h1 class="text-4xl font-bold tracking-tight">(t!(contact_title))</h1>
            <p class="mt-6 text-lg leading-relaxed text-stone-600">(t!(contact_text))</p>
            <p class="mt-8">
                <span class="block text-sm text-stone-500">(t!(contact_email_label))</span>
                <a class="text-xl font-semibold underline" href=(format!("mailto:{email}"))>
                    (email)
                </a>
            </p>
            <p class="mt-8 leading-relaxed text-stone-700">
                (t!(contact_data_text))
                " "
                <a class="underline" href=(href!(privacy::privacy, language: language(cx)?))>
                    (t!(contact_privacy_link))
                </a>
            </p>
        </main>
    })
}
