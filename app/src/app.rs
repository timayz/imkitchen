//! The pages: what a signed-in user reads ([`_signed_in`]: the home at `/`
//! and each screen of the application at the address of its module), what
//! a visitor who is not is shown ([`auth`], all of it under `/auth`), and
//! what is not a page ([`_app`], under `/_app`): the health, for the probes.
//!
//! The public pages of imkitchen are www's: app links to them where www
//! always answers ([`Www`]), the main domain being app for whoever is
//! signed in.
//!
//! Every page is read from the store of app, which its subscriptions fill
//! from the api's events: none asks the api for what it shows. The api is
//! asked who reads them, and handed what they do.
//!
//! A page is read in the language the browser prefers among those app
//! speaks: its address is the same in each. The sentences are in
//! `app/l10n`, one file per language: a language is spoken once its file is
//! there. No search engine is to list any page.

pub mod _app;
pub mod _signed_in;
pub mod auth;

use topcoat::{
    Result,
    asset::Asset,
    context::{BaseUrl, Cx, context},
    cookie::CookieLayer,
    l10n::{Locale, accept_language, locale, negotiate},
    metadata::{Metadata, Robots},
    router::{Body, Next, RouterBuilder, Slot, layer, layout, module_router, response::Response},
    tailwind,
    view::{View, view},
};

use crate::api::Api;
use crate::db::Db;
use crate::events::Events;
use crate::session::{Secure, Sessions};

topcoat::l10n::fluent::messages!(pub(crate) "{mod}/l10n/{locale}.ftl");

/// The stylesheet of every page, as `build.rs` compiles it.
pub const STYLESHEET: Asset = tailwind::stylesheet!();

/// The language of a reader whose browser asks for none app speaks.
const DEFAULT_LOCALE: Locale = locale!("en");

/// The address that always serves www, the public pages of imkitchen.
#[derive(Debug, Clone)]
pub struct Www(pub BaseUrl);

/// The pages of app, read from `db`, saying how app stands with the api
/// through `events`, asking `api` who reads them, keeping the sessions as
/// `sessions` says and linking to the public pages at `www`. Give the
/// result its assets, then build it.
pub fn router(db: Db, events: Events, api: Api, sessions: Sessions, www: Www) -> RouterBuilder {
    module_router!()
        .layer(CookieLayer::new())
        .with(db)
        .with(events)
        .with(api)
        .with(www)
        .with(sessions.key)
        .with(Secure(sessions.secure))
}

/// The read models of app.
pub(crate) fn db(cx: &Cx) -> &Db {
    context(cx)
}

/// The api, for who reads the pages and what they do.
pub(crate) fn api(cx: &Cx) -> &Api {
    context(cx)
}

/// The address of a public page: `page` is what follows the language in
/// it (`/terms`; nothing for the home). www has its pages under the
/// language they are in, and speaks those app does.
pub(crate) fn public(cx: &Cx, page: &str) -> String {
    let Www(www) = context(cx);
    www.join(&format!("{}{page}", language(cx)))
}

/// The language the page is read in, as the api is asked to word its
/// refusals: `fr`.
pub(crate) fn language(cx: &Cx) -> String {
    locale(cx).id().to_string()
}

/// Every page is read in the language the browser prefers among those app
/// speaks.
#[layer]
async fn preferred_language(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let preferred = negotiate(accept_language(cx), t::locales())
        .cloned()
        .unwrap_or(DEFAULT_LOCALE);
    next.run(&cx.with(preferred), body).await
}

#[layout(with(
    Metadata::new()
        .title(t!(site_title))
        .robots(Robots::new().index(false).follow(false))
))]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let current = locale(cx);

    Ok(view! {
        <!DOCTYPE html>
        <html lang=(current.id()) dir=(current.id().direction())>
            <head>
                topcoat::metadata::tags()
                <link rel="stylesheet" href=(STYLESHEET)>
                topcoat::dev::script()
            </head>
            <body class="min-h-dvh bg-stone-50 text-stone-900 antialiased">(slot)</body>
        </html>
    })
}
