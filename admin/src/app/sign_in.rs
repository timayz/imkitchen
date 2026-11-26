//! `/sign-in`: where an administrator says who they are.
//!
//! Signing in is the api's: the form posts an address and its password,
//! the api mails a code to the address, and the same again with the code
//! opens a session (root needs no code). The password is asked for at both
//! steps, as the api checks it at both, and is never written back into the
//! page. A session of someone who is not an administrator is ended as soon
//! as it is opened.

use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        StatusCode, content::Form, error::see_other, header::USER_AGENT, href, page, query_params,
        request::headers,
    },
    view::{View, component, view},
};

use super::{_admin, api};
use crate::api::{CommandError, Credentials, SignIn};
use crate::session::{self, Kept};

/// In the address of the page: someone signed in who is not an
/// administrator, and was signed out.
pub(crate) const REFUSED: &str = "refused";

/// What the form posts.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Posted {
    email: String,
    password: String,
    code: String,
}

#[query_params]
struct Query {
    refused: Option<String>,
}

/// How the page stands with who signs in.
enum Outcome {
    /// Nothing was posted yet.
    Blank,
    /// The password was right, and a code is on its way.
    CodeSent,
    /// Who signed in is not an administrator.
    NotAdmin,
    /// The api refused the sign-in, as it says.
    Refused { status: u16, message: String },
    /// The api could not be asked.
    Failed,
}

#[page]
pub async fn sign_in(cx: &Cx) -> Result<impl View> {
    let outcome = match query_params::<Query>(cx) {
        Ok(Query { refused: Some(_) }) => Outcome::NotAdmin,
        _ => Outcome::Blank,
    };

    Ok(view! { content(email: String::new(), asks_code: false, outcome: outcome) })
}

#[page(POST)]
async fn submit(cx: &Cx, Form(posted): Form<Posted>) -> Result<impl View> {
    let code = Some(posted.code.trim()).filter(|code| !code.is_empty());
    let credentials = Credentials {
        email: posted.email.trim().to_owned(),
        password: posted.password,
        code: code.map(str::to_owned),
    };
    let user_agent = headers(cx)
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok());

    let outcome = match api(cx).sign_in(&credentials, user_agent).await {
        Ok(SignIn::Done(session)) if session.user.is_admin() => {
            session::keep(
                cx,
                Kept {
                    token: session.token,
                    expires_at: session.expires_at,
                },
            )?;
            return Err(see_other(href!(_admin::dashboard).resolve(cx)).into());
        }
        Ok(SignIn::Done(session)) => {
            if let Err(err) = api(cx).sign_out(&session.token).await {
                tracing::warn!(err = format!("{err:#}"), "a session was not ended");
            }
            Outcome::NotAdmin
        }
        Ok(SignIn::CodeSent) => Outcome::CodeSent,
        Err(CommandError::Refused(refusal)) => Outcome::Refused {
            status: refusal.status,
            message: refusal.message,
        },
        Err(CommandError::Unreachable(err)) => {
            tracing::warn!(
                err = format!("{err:#}"),
                "a sign-in was not handed to the api"
            );
            Outcome::Failed
        }
    };
    // The code is asked for once one was mailed, and again when it was
    // wrong.
    let asks_code = match outcome {
        Outcome::CodeSent => true,
        Outcome::Refused { .. } | Outcome::Failed => credentials.code.is_some(),
        Outcome::Blank | Outcome::NotAdmin => false,
    };

    Ok(view! { content(email: credentials.email, asks_code: asks_code, outcome: outcome) })
}

/// The page: the form, over the `email` that was typed, with the field of
/// the code once one was mailed.
#[component]
async fn content(cx: &Cx, email: String, asks_code: bool, outcome: Outcome) -> Result<impl View> {
    let status = match &outcome {
        Outcome::Blank | Outcome::CodeSent => StatusCode::OK,
        Outcome::NotAdmin => StatusCode::FORBIDDEN,
        Outcome::Refused { status, .. } => {
            StatusCode::from_u16(*status).unwrap_or(StatusCode::BAD_GATEWAY)
        }
        Outcome::Failed => StatusCode::SERVICE_UNAVAILABLE,
    };
    let code_sent = matches!(outcome, Outcome::CodeSent);
    let refused = match outcome {
        Outcome::Blank | Outcome::CodeSent => None,
        Outcome::NotAdmin => Some((
            "not-admin",
            "This account is not an administrator's.".to_owned(),
        )),
        Outcome::Refused { message, .. } => Some(("refused", message)),
        Outcome::Failed => Some((
            "failed",
            "Signing in is not possible right now. Try again in a moment.".to_owned(),
        )),
    };
    let input = "mt-1 block w-full rounded-lg border border-stone-300 bg-white px-3 py-2";

    Ok(view! {
        (status)
        <main class="mx-auto max-w-sm px-4 py-16">
            <p class="text-xl font-semibold tracking-tight">"imkitchen admin"</p>
            <h1 class="mt-8 text-3xl font-bold tracking-tight">"Sign in"</h1>
            <form class="mt-8 grid gap-5" method="post" action=(href!(submit))>
                if let Some((what, message)) = refused {
                    <p
                        class="rounded-lg border border-red-300 bg-red-50 px-4 py-3 text-red-900"
                        role="alert"
                        data-sign-in=(what)
                    >
                        (message)
                    </p>
                }
                if code_sent {
                    <p
                        class="rounded-lg border border-green-300 bg-green-50 px-4 py-3 text-green-900"
                        role="status"
                        data-sign-in="code-sent"
                    >
                        "A code was mailed to this address. Type it below, with your password again."
                    </p>
                }
                <div>
                    <label class="text-sm font-medium" for="sign-in-email">"Email"</label>
                    <input
                        id="sign-in-email"
                        class=(input)
                        type="email"
                        name="email"
                        value=(email)
                        autocomplete="username"
                        required=""
                    >
                </div>
                <div>
                    <label class="text-sm font-medium" for="sign-in-password">"Password"</label>
                    <input
                        id="sign-in-password"
                        class=(input)
                        type="password"
                        name="password"
                        autocomplete="current-password"
                        required=""
                    >
                </div>
                if asks_code {
                    <div>
                        <label class="text-sm font-medium" for="sign-in-code">"Code"</label>
                        <input
                            id="sign-in-code"
                            class=(input)
                            type="text"
                            name="code"
                            inputmode="numeric"
                            autocomplete="one-time-code"
                            required=""
                        >
                    </div>
                }
                <div>
                    <button
                        class="rounded-lg bg-stone-900 px-5 py-3 font-semibold text-stone-50 hover:bg-stone-700"
                        type="submit"
                    >
                        "Sign in"
                    </button>
                </div>
            </form>
        </main>
    })
}
