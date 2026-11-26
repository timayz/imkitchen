//! The session of an administrator, as their browser keeps it.
//!
//! The api holds the sessions; the browser holds the token of its own, in a
//! private (encrypted, authenticated) cookie no script reads, for as long as
//! the api says the session lasts. Nothing of it is kept by admin.

use serde::{Deserialize, Serialize};
use timada_core::unix_now;
use topcoat::{
    context::{Cx, context},
    cookie::{Cookies, Key, SameSite, UnparsedCookieStore, cookie_store, private_cookies, time},
};

use crate::config::{ServerConfig, SessionConfig};

const COOKIE: &str = "session";

/// How the browsers keep their sessions.
pub struct Sessions {
    /// What the cookie is encrypted under.
    pub key: Key,
    /// Whether the cookie is only ever sent over HTTPS.
    pub secure: bool,
}

impl Sessions {
    /// From the configuration. Without a key, one is drawn now: fine for
    /// one admin alone, whose administrators then sign in again each time
    /// it starts.
    pub fn new(session: &SessionConfig, server: &ServerConfig) -> Self {
        let key = match &session.key {
            Some(key) => Key::derive_from(key.expose().as_bytes()),
            None => {
                tracing::warn!(
                    "session.key is not set: whoever is signed in signs in again after a restart"
                );
                Key::generate()
            }
        };
        Self {
            key,
            secure: server.secure(),
        }
    }
}

/// Whether the cookie is only ever sent over HTTPS, for the pages.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Secure(pub bool);

/// What a browser keeps of its session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kept {
    pub token: String,
    /// Unix seconds: when the api ends the session, if it is not used.
    pub expires_at: u64,
}

/// The cookie, kept for `max_age_secs` when it is written.
fn store(cx: &Cx, max_age_secs: u64) -> UnparsedCookieStore<Kept, impl Cookies + '_> {
    let max_age = i64::try_from(max_age_secs).unwrap_or(i64::MAX);
    cookie_store(
        private_cookies(cx)
            .default_same_site(SameSite::Lax)
            .default_path("/")
            .default_http_only(true)
            .default_secure(context::<Secure>(cx).0)
            .default_max_age(time::Duration::seconds(max_age)),
        COOKIE,
    )
}

/// The session this browser keeps, if any: one the api may have ended.
pub fn kept(cx: &Cx) -> Option<Kept> {
    Some(store(cx, 0).parse().ok()??.get())
}

/// Keeps `session` on this browser until the api ends it.
pub fn keep(cx: &Cx, session: Kept) -> topcoat::Result<()> {
    store(cx, session.expires_at.saturating_sub(unix_now()))
        .set(session)
        .commit()?;
    Ok(())
}

/// Forgets the session this browser keeps.
pub fn forget(cx: &Cx) {
    store(cx, 0).remove();
}
