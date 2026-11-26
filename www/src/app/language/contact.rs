//! `/{language}/contact`: how a visitor writes to imkitchen.
//!
//! The form is a command of the api: www keeps nothing of it. What the api
//! refuses of it is shown where the visitor corrects it, as the api words
//! it, over what they typed.

use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    l10n::locale,
    metadata::Metadata,
    router::{StatusCode, content::Form, error::see_other, href, page, query_params},
    view::{View, class, component, view},
};

use super::{language, privacy};
use crate::api::{CommandError, ContactMessage, Refusal};
use crate::app::{api, site, t};

/// What the form posts.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Posted {
    name: String,
    email: String,
    message: String,
    /// A field no person sees, and so none fills: what does is not a
    /// person, and its message goes nowhere.
    website: String,
}

/// After a message was taken, the page says so in place of the form.
#[query_params]
struct Query {
    sent: Option<String>,
}

/// How the page stands with the message of the visitor.
enum Outcome {
    /// None was written yet.
    Blank,
    /// It was taken.
    Sent,
    /// The api refused it, and says why.
    Refused(Refusal),
    /// It could not be handed to the api.
    Failed,
}

#[page(with(Metadata::new().title(t!(contact_meta_title)).description(t!(contact_description))))]
pub async fn contact(cx: &Cx) -> Result<impl View> {
    let outcome = match query_params::<Query>(cx) {
        Ok(Query { sent: Some(_) }) => Outcome::Sent,
        _ => Outcome::Blank,
    };

    Ok(view! { content(written: ContactMessage::default(), outcome: outcome) })
}

#[page(POST, with(Metadata::new().title(t!(contact_meta_title)).description(t!(contact_description))))]
async fn send(cx: &Cx, Form(posted): Form<Posted>) -> Result<impl View> {
    let sent = href!(contact, language: language(cx)?).query([("sent", "1")]);
    if !posted.website.is_empty() {
        return Err(see_other(sent.resolve(cx)).into());
    }
    let written = ContactMessage {
        name: posted.name,
        email: posted.email,
        message: posted.message,
    };
    let outcome = match api(cx).send_contact_message(locale(cx), &written).await {
        // Read again, the page does not send the message again.
        Ok(()) => return Err(see_other(sent.resolve(cx)).into()),
        Err(CommandError::Refused(refusal)) => Outcome::Refused(refusal),
        Err(CommandError::Unreachable(err)) => {
            tracing::warn!(err = format!("{err:#}"), "a contact message was not sent");
            Outcome::Failed
        }
    };

    Ok(view! { content(written: written, outcome: outcome) })
}

/// The page: the form, over what was `written` when it comes back refused,
/// or that the message was taken.
#[component]
async fn content(cx: &Cx, written: ContactMessage, outcome: Outcome) -> Result<impl View> {
    let email = &site(cx).contact.email;
    let (status, refusal) = match &outcome {
        Outcome::Blank | Outcome::Sent => (StatusCode::OK, None),
        Outcome::Refused(refusal) => (
            StatusCode::from_u16(refusal.status).unwrap_or(StatusCode::BAD_GATEWAY),
            Some(refusal),
        ),
        Outcome::Failed => (StatusCode::SERVICE_UNAVAILABLE, None),
    };
    let refused = |field: &str| {
        refusal
            .and_then(|refusal| refusal.field(field))
            .map(str::to_owned)
    };
    let (name, address, message) = (refused("name"), refused("email"), refused("message"));
    // What the api refuses that no field of the form answers for.
    let refused_whole = refusal
        .filter(|_| name.is_none() && address.is_none() && message.is_none())
        .map(|refusal| refusal.message.clone());
    let input = "mt-1 block w-full rounded-lg border bg-white px-3 py-2";

    Ok(view! {
        (status)
        <main class="mx-auto max-w-3xl px-4 pt-12 pb-16">
            <h1 class="text-4xl font-bold tracking-tight">(t!(contact_title))</h1>
            <p class="mt-6 text-lg leading-relaxed text-stone-600">(t!(contact_text))</p>
            if matches!(outcome, Outcome::Sent) {
                <p
                    class="mt-8 rounded-lg border border-green-300 bg-green-50 px-4 py-3 text-green-900"
                    role="status"
                    data-contact="sent"
                >
                    (t!(contact_sent))
                </p>
            } else {
                <form
                    class="mt-8 grid gap-5"
                    method="post"
                    action=(href!(send, language: language(cx)?))
                >
                    if matches!(outcome, Outcome::Failed) {
                        <p
                            class="rounded-lg border border-red-300 bg-red-50 px-4 py-3 text-red-900"
                            role="alert"
                            data-contact="failed"
                        >
                            (t!(contact_failed))
                        </p>
                    }
                    if let Some(refused_whole) = refused_whole {
                        <p
                            class="rounded-lg border border-red-300 bg-red-50 px-4 py-3 text-red-900"
                            role="alert"
                            data-contact="refused"
                        >
                            (refused_whole)
                        </p>
                    }
                    <div>
                        <label class="text-sm font-medium" for="contact-name">
                            (t!(contact_name))
                        </label>
                        <input
                            id="contact-name"
                            class=(class!(input, "border-stone-300" if name.is_none(), "border-red-500" if name.is_some()))
                            type="text"
                            name="name"
                            value=(written.name)
                            autocomplete="name"
                            maxlength="100"
                            required=""
                            aria-invalid=(name.is_some().then_some("true"))
                            aria-describedby=(name.is_some().then_some("contact-name-error"))
                        >
                        if let Some(name) = name {
                            <p id="contact-name-error" class="mt-1 text-sm text-red-700">(name)</p>
                        }
                    </div>
                    <div>
                        <label class="text-sm font-medium" for="contact-email">
                            (t!(contact_email))
                        </label>
                        <input
                            id="contact-email"
                            class=(class!(input, "border-stone-300" if address.is_none(), "border-red-500" if address.is_some()))
                            type="email"
                            name="email"
                            value=(written.email)
                            autocomplete="email"
                            required=""
                            aria-invalid=(address.is_some().then_some("true"))
                            aria-describedby=(address.is_some().then_some("contact-email-error"))
                        >
                        if let Some(address) = address {
                            <p id="contact-email-error" class="mt-1 text-sm text-red-700">
                                (address)
                            </p>
                        }
                    </div>
                    <div>
                        <label class="text-sm font-medium" for="contact-message">
                            (t!(contact_message))
                        </label>
                        <textarea
                            id="contact-message"
                            class=(class!(input, "border-stone-300" if message.is_none(), "border-red-500" if message.is_some()))
                            name="message"
                            rows="8"
                            maxlength="4000"
                            required=""
                            aria-invalid=(message.is_some().then_some("true"))
                            aria-describedby=(message.is_some().then_some("contact-message-error"))
                        >
                            (written.message)
                        </textarea>
                        if let Some(message) = message {
                            <p id="contact-message-error" class="mt-1 text-sm text-red-700">
                                (message)
                            </p>
                        }
                    </div>
                    <div class="hidden" aria-hidden="true">
                        <label for="contact-website">"Website"</label>
                        <input
                            id="contact-website"
                            type="text"
                            name="website"
                            tabindex="-1"
                            autocomplete="off"
                        >
                    </div>
                    <div>
                        <button
                            class="rounded-lg bg-stone-900 px-5 py-3 font-semibold text-stone-50 hover:bg-stone-700"
                            type="submit"
                        >
                            (t!(contact_send))
                        </button>
                    </div>
                </form>
            }
            <p class="mt-10">
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
