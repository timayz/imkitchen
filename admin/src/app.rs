//! The pages: `/` is the dashboard, `/sign-in` is where an administrator
//! says who they are, and `/health` is for the probes.
//!
//! Only an administrator reads admin: every page is declared under
//! [`_admin`], which lets nobody else through, but the one that signs in.
//!
//! admin is read by the people who run imkitchen, in English: its sentences
//! are in the pages. No search engine is to list any of them.

pub mod _admin;
pub mod health;
pub mod sign_in;

use topcoat::{
    Result,
    asset::Asset,
    context::{Cx, context},
    cookie::CookieLayer,
    metadata::Metadata,
    router::{RouterBuilder, Slot, error::see_other, href, layout, module_router, route},
    tailwind,
    view::{View, view},
};

use crate::api::Api;
use crate::db::Db;
use crate::events::{Events, Executor};
use crate::session::{self, Secure, Sessions};

/// The stylesheet of every page, as `build.rs` compiles it.
pub const STYLESHEET: Asset = tailwind::stylesheet!();

/// The pages of admin, read from `db`, saying how admin stands with the
/// api through `events`, asking `api` who reads them and keeping the
/// sessions as `sessions` says. Give the result its assets, then build it.
pub fn router(db: Db, events: Events, api: Api, sessions: Sessions) -> RouterBuilder {
    module_router!()
        .layer(CookieLayer::new())
        .with(db)
        .with(events)
        .with(api)
        .with(sessions.key)
        .with(Secure(sessions.secure))
}

/// The read models of admin.
pub(crate) fn db(cx: &Cx) -> &Db {
    context(cx)
}

/// The api's events, which a page loads one recipe or picture from: past
/// what the read models hold, up to the command an administrator just
/// handed the api.
pub(crate) fn events(cx: &Cx) -> &Executor {
    context::<Events>(cx).executor()
}

/// The api, for who reads the pages.
pub(crate) fn api(cx: &Cx) -> &Api {
    context(cx)
}

#[layout(with(Metadata::new().title("imkitchen admin")))]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                topcoat::metadata::tags()
                <meta name="robots" content="noindex, nofollow">
                <link rel="stylesheet" href=(STYLESHEET)>
                topcoat::dev::script()
            </head>
            <body class="min-h-screen bg-stone-50 text-stone-900 antialiased">(slot)</body>
        </html>
    })
}

/// Ends the session of the browser, at the api and on it. Whoever asks is
/// signed out, the api answering or not: a session it could not be told to
/// end lasts until it expires, with nothing left that opens it.
#[route(POST "./sign-out")]
async fn sign_out(cx: &Cx) -> Result<()> {
    if let Some(kept) = session::kept(cx) {
        if let Err(err) = api(cx).sign_out(&kept.token).await {
            tracing::warn!(err = format!("{err:#}"), "a session was not ended");
        }
        session::forget(cx);
    }
    Err(see_other(href!(sign_in::sign_in).resolve(cx)).into())
}
