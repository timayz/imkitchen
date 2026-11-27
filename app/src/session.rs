//! The session of a user, as their browser keeps it.
//!
//! The api holds the sessions; the browser holds the token of its own, in a
//! private (encrypted, authenticated) cookie no script reads, for as long as
//! the api says the session lasts: closing the browser does not end it.
//! Nothing of it is kept by app.
//!
//! Beside it goes a second cookie, the marker ([`MARKER`]): it says a
//! session is there and nothing else. It is what the gateway in front of
//! imkitchen routes the main domain on, to app when it is there and to www
//! when it is not, without being able to read the first. The two are
//! written together and taken back together; the marker alone opens
//! nothing.

use serde::{Deserialize, Serialize};
use timada_core::unix_now;
use topcoat::{
    context::{Cx, context},
    cookie::{
        Cookie, Cookies, Key, SameSite, UnparsedCookieStore, cookie_store, cookies,
        private_cookies, time,
    },
};

use crate::config::{ServerConfig, SessionConfig};

const COOKIE: &str = "session";

/// The name of the marker cookie: what the gateway matches.
pub const MARKER: &str = "signed_in";

/// What the marker holds. Its being there is all it says.
const MARKED: &str = "1";

/// How the browsers keep their sessions.
pub struct Sessions {
    /// What the cookie is encrypted under.
    pub key: Key,
    /// Whether the cookies are only ever sent over HTTPS.
    pub secure: bool,
}

impl Sessions {
    /// From the configuration. Without a key, one is drawn now: fine for
    /// one app alone, whose users then sign in again each time it starts.
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

/// Whether the cookies are only ever sent over HTTPS, for the pages.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Secure(pub bool);

/// What a browser keeps of its session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kept {
    pub token: String,
    /// Unix seconds: when the api ends the session, if it is not used.
    pub expires_at: u64,
}

/// How the two cookies are written over `jar`, kept for `max_age_secs`.
fn written<'a>(cx: &Cx, jar: impl Cookies + 'a, max_age_secs: u64) -> impl Cookies + 'a {
    let max_age = i64::try_from(max_age_secs).unwrap_or(i64::MAX);
    jar.default_same_site(SameSite::Lax)
        .default_path("/")
        .default_http_only(true)
        .default_secure(context::<Secure>(cx).0)
        .default_max_age(time::Duration::seconds(max_age))
}

/// The cookie of the session, kept for `max_age_secs` when it is written.
fn store(cx: &Cx, max_age_secs: u64) -> UnparsedCookieStore<Kept, impl Cookies + '_> {
    cookie_store(written(cx, private_cookies(cx), max_age_secs), COOKIE)
}

/// The session this browser keeps, if any: one the api may have ended.
pub fn kept(cx: &Cx) -> Option<Kept> {
    Some(store(cx, 0).parse().ok()??.get())
}

/// Whether this browser says it keeps a session, with one or with the
/// marker: what there is to take back when the api knows none.
pub fn claimed(cx: &Cx) -> bool {
    cookies(cx).get(COOKIE).is_some() || marked(cx)
}

/// Whether this browser keeps the marker.
pub fn marked(cx: &Cx) -> bool {
    cookies(cx).get(MARKER).is_some()
}

/// Keeps `session` on this browser until the api ends it, the marker beside
/// it for as long.
pub fn keep(cx: &Cx, session: Kept) -> topcoat::Result<()> {
    let lasts = session.expires_at.saturating_sub(unix_now());
    store(cx, lasts).set(session).commit()?;
    written(cx, cookies(cx), lasts).add(Cookie::new(MARKER, MARKED));
    Ok(())
}

/// Forgets the session this browser keeps, and the marker with it.
pub fn forget(cx: &Cx) {
    store(cx, 0).remove();
    written(cx, cookies(cx), 0).remove(Cookie::from(MARKER));
}
