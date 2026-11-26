//! What an administrator reads: every page of admin but the one that signs
//! in. `/` is the dashboard.
//!
//! The module adds nothing to the addresses of its pages (its name starts
//! with `_`). What it adds is who may read them: whatever is declared here
//! or below is behind [`administrators_only`].

use topcoat::{
    Error, Result,
    context::{Cx, context},
    router::{
        Body, Next, Slot,
        error::{NotFoundError, see_other, service_unavailable},
        href, layer, layout, not_found, page,
        response::Response,
    },
    view::{View, error_boundary, view},
};

use super::{api, db, sign_in, sign_out};
use crate::api::Account;
use crate::session::{self, Kept};
use crate::users;

/// How long a browser waits before it asks again, when the api could not
/// say who it is: in seconds.
const RETRY_AFTER_SECS: u64 = 5;

// Anything else is the "not found" page, of an administrator's too.
not_found!();

/// Who reads the page.
pub(crate) fn account(cx: &Cx) -> &Account {
    context(cx)
}

/// Lets an administrator through, and nobody else: the api is asked whose
/// session the browser keeps on every request, so one that ended, or whose
/// user is an administrator no more, reads nothing further.
#[layer]
async fn administrators_only(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let sign_in = href!(sign_in::sign_in);
    let Some(kept) = session::kept(cx) else {
        return Err(see_other(sign_in.resolve(cx)).into());
    };
    let me = match api(cx).account(&kept.token).await {
        Ok(Some(me)) => me,
        Ok(None) => {
            session::forget(cx);
            return Err(see_other(sign_in.resolve(cx)).into());
        }
        Err(err) => {
            tracing::warn!(
                err = format!("{err:#}"),
                "the api does not say who reads admin"
            );
            return Err(service_unavailable(RETRY_AFTER_SECS).into());
        }
    };
    if !me.user.is_admin() {
        // Its session was opened for admin, which has no use for it.
        if let Err(err) = api(cx).sign_out(&kept.token).await {
            tracing::warn!(err = format!("{err:#}"), "a session was not ended");
        }
        session::forget(cx);
        let refused = sign_in.query([(sign_in::REFUSED, "1")]);
        return Err(see_other(refused.resolve(cx)).into());
    }
    // The api extends a session that is used: so does the browser.
    if me.expires_at > kept.expires_at {
        session::keep(
            cx,
            Kept {
                expires_at: me.expires_at,
                ..kept
            },
        )?;
    }
    next.run(&cx.with(me.user), body).await
}

#[layout]
async fn signed_in(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let account = account(cx);

    Ok(view! {
        <header class="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-x-6 gap-y-2 px-4 py-6">
            <a class="text-xl font-semibold tracking-tight" href=(href!(dashboard))>
                "imkitchen admin"
            </a>
            <div class="flex items-center gap-4 text-sm">
                <span class="text-stone-600" data-account=(account.email.as_str())>
                    (account.email.as_str())
                </span>
                <form method="post" action=(href!(sign_out))>
                    <button class="font-medium underline hover:text-stone-600" type="submit">
                        "Sign out"
                    </button>
                </form>
            </div>
        </header>
        error_boundary(
            fallback: |error| {
                if error.downcast_ref::<NotFoundError>().is_none() {
                    return Err(error);
                }

                Ok(
                    view! {
                        (topcoat::router::StatusCode::NOT_FOUND)
                        <main class="mx-auto max-w-5xl px-4 py-16">
                            <h1 class="text-3xl font-bold">"This page does not exist"</h1>
                            <p class="mt-4 text-stone-600">
                                "The address may have changed, or never was one."
                            </p>
                        </main>
                    },
                )
            },
            (slot)
        )
    })
}

#[page]
pub async fn dashboard(cx: &Cx) -> Result<impl View> {
    let users = users::count(db(cx)).await.map_err(Error::from_anyhow)?;

    Ok(view! {
        <main class="mx-auto max-w-5xl px-4 py-10">
            <h1 class="text-3xl font-bold tracking-tight">"Dashboard"</h1>
            <dl class="mt-8 grid gap-4 sm:grid-cols-3">
                <div class="rounded-lg border border-stone-200 bg-white p-6">
                    <dt class="text-sm text-stone-500">"Users"</dt>
                    <dd class="mt-2 text-3xl font-semibold" data-users=(users)>(users)</dd>
                </div>
            </dl>
        </main>
    })
}
