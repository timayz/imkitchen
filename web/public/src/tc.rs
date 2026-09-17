//! SPIKE: topcoat login slice, served at `/_tc/login` next to the askama one.
//!
//! Proves the fixed-field form pattern: per-field signals + a `#[procedure]`
//! that returns its outcome as data, an in-place toast on failure, and the
//! plain `action="/login"` POST as the no-JS fallback.

use imkitchen_identity::LoginInput;
use imkitchen_web_shared::tc::{
    auth::{require_user, set_auth_cookie},
    context::{i18n, language_iso, state, timezone},
    error::{Outcome, server_toast_message, toastify},
    view::{base, streamed},
};
use topcoat::{
    Result,
    context::Cx,
    router::{RouterBuilder, page, request::headers},
    runtime::{Event, RouterBuilderProcedureExt, procedure, signal},
    view::{View, emit, live, view},
};

/// Logs in and queues the `auth_token` cookie on the procedure's own response.
/// `Ok(path)` is where the browser should go next; `Err(text)` is a toast.
///
/// No guard: this is the anonymous entry point. Inputs are untrusted and go
/// through the same `LoginInput` validation as the askama handler.
#[procedure]
async fn login(cx: &Cx, email: String, password: String) -> Result<Outcome<String>> {
    let user_agent = headers(cx)
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();

    let logged_in = state(cx)
        .identity
        .login(LoginInput {
            email,
            password,
            lang: language_iso(cx).to_owned(),
            timezone: timezone(cx).to_owned(),
            user_agent,
        })
        .await;

    let (user_id, access_id) = match toastify(cx, logged_in) {
        Ok(ids) => ids,
        Err(message) => return Ok(Err(message)),
    };

    if let Err(err) = set_auth_cookie(cx, user_id, access_id) {
        return Ok(Err(server_toast_message(cx, &err)));
    }

    Ok(Ok("/".to_owned()))
}

#[page("/_tc/login")]
async fn login_page(cx: &Cx) -> Result<impl View> {
    let t = i18n(cx);
    let email = signal(cx, String::new);
    let password = signal(cx, String::new);
    let pending = signal(cx, || false);

    Ok(view! {
        base(
            title: t.t("Log In"),
            <div class="container mx-auto px-4 py-16 max-w-md">
                <div class="bg-paper rounded-2xl shadow-md border border-line-2 p-8">
                    <h1 class="text-3xl font-serif font-bold mb-2 text-center text-ink">(t.t("Welcome Back"))</h1>
                    <p class="text-ink-2 text-center mb-8">(t.t("Log in to access your meal plans"))</p>

                    <form
                        action="/login"
                        method="post"
                        @submit=$(async |e: Event| {
                            e.prevent_default();
                            pending.set(true);
                            let outcome = login(email.get(), password.get()).await;
                            pending.set(false);
                            if outcome.is_ok() {
                                let to = outcome.unwrap();
                                raw!("location.assign(${to})", drop(to));
                            } else {
                                let message = outcome.unwrap_err();
                                raw!("imk.toast('error', ${message})", drop(message));
                            }
                        })
                    >
                        <div class="mb-6">
                            <label for="email" class="block text-sm font-semibold text-ink-2 mb-2">(t.t("Email Address"))</label>
                            <input
                                type="email"
                                id="email"
                                name="email"
                                placeholder=(t.t("you@example.com"))
                                class="w-full px-4 py-3 border border-line rounded-xl focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                                autocomplete="new-email"
                                required=(true)
                                :value=$(email.get())
                                @input=$(|e: Event| email.set(e.target.value))
                            >
                        </div>

                        <div class="mb-6">
                            <label for="password" class="block text-sm font-semibold text-ink-2 mb-2">(t.t("Password"))</label>
                            <input
                                type="password"
                                id="password"
                                name="password"
                                placeholder="••••••••"
                                class="w-full px-4 py-3 border border-line rounded-xl focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                                autocomplete="new-password"
                                required=(true)
                                :value=$(password.get())
                                @input=$(|e: Event| password.set(e.target.value))
                            >
                        </div>

                        <div class="flex items-center justify-between mb-6">
                            <div class="flex items-center"></div>
                            <a href="/reset-password" class="text-sm text-primary-500 hover:text-primary-600">(t.t("Forgot password?"))</a>
                        </div>

                        <button
                            type="submit"
                            class="w-full bg-primary-500 text-white font-semibold py-3 rounded-xl hover:bg-primary-600 transition cursor-pointer disabled:opacity-60"
                            :disabled=$(pending.get())
                        >
                            (t.t("Log In"))
                        </button>
                    </form>

                    <div class="mt-6 text-center">
                        <p class="text-ink-2 text-sm">
                            (t.t("Don't have an account?")) " "
                            <a href="/register" class="text-primary-500 font-semibold hover:text-primary-600">(t.t("Sign up free"))</a>
                        </p>
                    </div>
                </div>
            </div>
        )
    })
}

/// SPIKE: guarded page proving a session issued by either stack authenticates
/// on the other (same `auth_token` cookie, same exact `User-Agent` match).
#[page("/_tc/me")]
async fn me_page(cx: &Cx) -> Result<impl View> {
    let user = require_user(cx).await?;
    let email = user.email.to_owned();
    let tz = timezone(cx).to_owned();

    Ok(view! {
        base(
            title: "Me",
            <main class="container mx-auto px-4 py-16">
                <p id="me-email">(email)</p>
                <p id="me-tz">(tz)</p>
            </main>
        )
    })
}

/// SPIKE: finite `live!` region, the replacement for on-load polling (billing
/// re-check, import progress). Proves the axum host streams topcoat responses
/// chunk by chunk, and that the page hydrates once the stream ends (#390 only
/// bites never-ending regions).
#[page("/_tc/live")]
async fn live_page(cx: &Cx) -> Result<impl View> {
    let clicks = signal(cx, || 0.0);

    Ok(view! {
        (streamed())
        base(
            title: "Live",
            <main class="container mx-auto px-4 py-16">
                <button id="live-button" type="button" @click=$(|_e| clicks.increment())>"clicked " $(clicks.get())</button>
                (live! {
                    for step in 1..=4 {
                        emit! { <p id="live-progress">"step " (step) " / 4"</p> }?;
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                    }
                    emit! { <p id="live-progress">"done"</p> }
                })
            </main>
        )
    })
}

pub fn tc_routes(builder: RouterBuilder) -> RouterBuilder {
    builder
        .page(login_page)
        .page(me_page)
        .page(live_page)
        .procedure(login)
}
