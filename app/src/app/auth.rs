//! `/auth`: what app shows a visitor who is not signed in, and where a
//! session starts and ends. `/auth/sign-in` and `/auth/register` are the
//! two pages, `POST /auth/sign-out` ends the session.
//!
//! Everything a visitor is shown is under this one prefix: it is what the
//! gateway in front of imkitchen routes to app whether the browser keeps
//! the marker of a session or not. The pages lead back to the public ones,
//! which are www's.
//!
//! Registering and signing in are the api's, and take the same two steps:
//! the form posts an address and its password, the api mails a code to the
//! address, and the same again with the code opens a session. The password
//! is asked for at both steps, as the api checks it at both, and is never
//! written back into the page. What the api refuses is said as it words it,
//! in the language the page is read in, a field that is not valid under
//! the field.

pub mod register;
pub mod sign_in;

use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    l10n::Translation,
    router::{
        Slot, StatusCode, error::see_other, header::USER_AGENT, href, layout, request::headers,
        route,
    },
    view::{View, component, view},
};

use super::{_signed_in, api, language, public, t};
use crate::api::{CommandError, Credentials, Refusal, SignIn};
use crate::session::{self, Kept};

/// What the two forms post.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct Posted {
    email: String,
    password: String,
    code: String,
}

/// Which of the two a form does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Doing {
    SigningIn,
    Registering,
}

/// How a page stands with who fills its form.
pub(crate) enum Outcome {
    /// Nothing was posted yet.
    Blank,
    /// The api took the address and the password, and a code is on its way.
    CodeSent,
    /// The api refused, as it says.
    Refused(Refusal),
    /// The api could not be asked.
    Failed,
}

/// A form as it is shown again: what was typed of it, and how it went.
pub(crate) struct Filled {
    email: String,
    /// Whether the code is asked for: once one was mailed, and again when
    /// it was wrong.
    asks_code: bool,
    outcome: Outcome,
}

impl Filled {
    pub(crate) fn blank() -> Self {
        Self {
            email: String::new(),
            asks_code: false,
            outcome: Outcome::Blank,
        }
    }
}

/// Sends a browser that is signed in already to the home: it has nothing
/// to do on these pages. A session the api ended is found out there.
pub(crate) fn visitors_only(cx: &Cx) -> Result<()> {
    if session::kept(cx).is_some() {
        return Err(see_other(href!(_signed_in::home).resolve(cx)).into());
    }
    Ok(())
}

/// Hands what a form posted to the api. A session opened is kept on the
/// browser, which is sent to the home; anything else is the form again.
pub(crate) async fn attempt(cx: &Cx, doing: Doing, posted: Posted) -> Result<Filled> {
    let code = Some(posted.code.trim()).filter(|code| !code.is_empty());
    let credentials = Credentials {
        email: posted.email.trim().to_owned(),
        password: posted.password,
        code: code.map(str::to_owned),
    };
    let user_agent = headers(cx)
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let language = language(cx);
    let asked = match doing {
        Doing::SigningIn => api(cx).sign_in(&credentials, user_agent, &language).await,
        Doing::Registering => api(cx).register(&credentials, user_agent, &language).await,
    };

    let outcome = match asked {
        Ok(SignIn::Done(opened)) => {
            session::keep(
                cx,
                Kept {
                    token: opened.token,
                    expires_at: opened.expires_at,
                },
            )?;
            return Err(see_other(href!(_signed_in::home).resolve(cx)).into());
        }
        Ok(SignIn::CodeSent) => Outcome::CodeSent,
        Err(CommandError::Refused(refusal)) => Outcome::Refused(refusal),
        Err(CommandError::Unreachable(err)) => {
            tracing::warn!(err = format!("{err:#}"), "a form was not handed to the api");
            Outcome::Failed
        }
    };
    let asks_code = match outcome {
        Outcome::CodeSent => true,
        Outcome::Refused(_) | Outcome::Failed => credentials.code.is_some(),
        Outcome::Blank => false,
    };

    Ok(Filled {
        email: credentials.email,
        asks_code,
        outcome,
    })
}

/// Ends the session of the browser, at the api and on it, and sends it to
/// the main domain: with the marker gone, what the gateway serves there is
/// www. Whoever asks is signed out, the api answering or not: a session it
/// could not be told to end lasts until it expires, with nothing left that
/// opens it.
#[route(POST "./sign-out")]
pub(crate) async fn sign_out(cx: &Cx) -> Result<()> {
    if let Some(kept) = session::kept(cx)
        && let Err(err) = api(cx).sign_out(&kept.token).await
    {
        tracing::warn!(err = format!("{err:#}"), "a session was not ended");
    }
    Err(_signed_in::signed_out(cx))
}

#[layout]
async fn visitor(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-sm px-4 py-12 md:py-20">
            <p class="text-xl font-semibold tracking-tight">
                <a href=(public(cx, "")) aria-label=(t!(public_about))>"imkitchen"</a>
            </p>
            (slot)
            <nav
                class="mt-12 flex gap-4 border-t border-stone-200 pt-6 text-sm text-stone-500"
                aria-label=(t!(public_pages))
            >
                <a class="hover:underline" href=(public(cx, "/terms"))>(t!(public_terms))</a>
                <a class="hover:underline" href=(public(cx, "/privacy"))>(t!(public_privacy))</a>
            </nav>
        </main>
    })
}

/// One of the two forms: its `title`, where it posts (`action`), what its
/// button says (`submit`), and the way to the other (`other`, behind
/// `other_label`).
#[component]
pub(crate) async fn form(
    cx: &Cx,
    doing: Doing,
    filled: Filled,
    title: Translation<'static>,
    submit: Translation<'static>,
    action: String,
    other: String,
    other_label: Translation<'static>,
) -> Result<impl View> {
    let Filled {
        email,
        asks_code,
        outcome,
    } = filled;
    let status = match &outcome {
        Outcome::Blank | Outcome::CodeSent => StatusCode::OK,
        Outcome::Refused(refusal) => {
            StatusCode::from_u16(refusal.status).unwrap_or(StatusCode::BAD_GATEWAY)
        }
        Outcome::Failed => StatusCode::SERVICE_UNAVAILABLE,
    };
    let code_sent = matches!(outcome, Outcome::CodeSent);
    let failed = matches!(outcome, Outcome::Failed);
    // What the api says of the whole, and of each field it names.
    let (refused, email_refused, password_refused, code_refused) = match outcome {
        Outcome::Refused(refusal) => (
            Some((refusal.code.clone(), refusal.message.clone())),
            refusal.field("email").map(str::to_owned),
            refusal.field("password").map(str::to_owned),
            refusal.field("code").map(str::to_owned),
        ),
        Outcome::Blank | Outcome::CodeSent | Outcome::Failed => (None, None, None, None),
    };
    let password_autocomplete = match doing {
        Doing::SigningIn => "current-password",
        Doing::Registering => "new-password",
    };
    let input = "mt-1 block w-full rounded-lg border border-stone-300 bg-white px-3 py-2";
    let label = "text-sm font-medium";
    let field_refused = "mt-1 text-sm text-red-800";

    Ok(view! {
        (status)
        <h1 class="mt-8 text-3xl font-bold tracking-tight">(title)</h1>
        <form class="mt-8 grid gap-5" method="post" action=(action)>
            if let Some((code, message)) = refused {
                <p
                    class="rounded-lg border border-red-300 bg-red-50 px-4 py-3 text-red-900"
                    role="alert"
                    data-auth="refused"
                    data-code=(code)
                >
                    (message)
                </p>
            }
            if failed {
                <p
                    class="rounded-lg border border-red-300 bg-red-50 px-4 py-3 text-red-900"
                    role="alert"
                    data-auth="failed"
                >
                    (t!(auth_failed))
                </p>
            }
            if code_sent {
                <p
                    class="rounded-lg border border-green-300 bg-green-50 px-4 py-3 text-green-900"
                    role="status"
                    data-auth="code-sent"
                >
                    (t!(auth_code_sent))
                </p>
            }
            <div>
                <label class=(label) for="auth-email">(t!(auth_email))</label>
                <input
                    id="auth-email"
                    class=(input)
                    type="email"
                    name="email"
                    value=(email)
                    autocomplete="username"
                    aria-invalid=(email_refused.is_some().then_some("true"))
                    required=""
                >
                if let Some(message) = email_refused {
                    <p class=(field_refused) data-field="email">(message)</p>
                }
            </div>
            <div>
                <label class=(label) for="auth-password">(t!(auth_password))</label>
                <input
                    id="auth-password"
                    class=(input)
                    type="password"
                    name="password"
                    autocomplete=(password_autocomplete)
                    aria-invalid=(password_refused.is_some().then_some("true"))
                    required=""
                >
                if let Some(message) = password_refused {
                    <p class=(field_refused) data-field="password">(message)</p>
                }
            </div>
            if asks_code {
                <div>
                    <label class=(label) for="auth-code">(t!(auth_code))</label>
                    <input
                        id="auth-code"
                        class=(input)
                        type="text"
                        name="code"
                        inputmode="numeric"
                        autocomplete="one-time-code"
                        aria-invalid=(code_refused.is_some().then_some("true"))
                        required=""
                    >
                    if let Some(message) = code_refused {
                        <p class=(field_refused) data-field="code">(message)</p>
                    }
                </div>
            }
            <div>
                <button
                    class="w-full rounded-lg bg-stone-900 px-5 py-3 font-semibold text-stone-50 hover:bg-stone-700"
                    type="submit"
                >
                    (submit)
                </button>
            </div>
        </form>
        <p class="mt-8 text-sm text-stone-600">
            <a class="font-medium underline hover:text-stone-900" href=(other)>(other_label)</a>
        </p>
    })
}
