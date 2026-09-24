use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};

use crate::{
    config::Config,
    template::{NotFoundTemplate, Template},
};

/// Appended to the stored User-Agent of a session opened inside the Android
/// shell. Deliberately carries no version: sessions are matched on the
/// normalized User-Agent key, so a versioned token would still compare equal
/// after `stable_ua_key` strips it — but a bare token keeps that independent
/// of the normalizer, and versioning a client token has historically been the
/// way to log an entire app's users out on update.
///
/// A TWA shares Chrome's User-Agent, so the shell cannot append this itself.
/// The server synthesizes it from the `Host` header at login instead. The
/// future iOS shell *can* append a real token and will land in this same code
/// path with no server change.
pub const NATIVE_UA_MARKER: &str = "imkitchen-twa";

/// True when this request was served on the dedicated native-app host.
///
/// This is the only signal that is correct on *every* request — navigations,
/// TwinSpark partials and form POSTs alike. Launch-only signals (an
/// `android-app://` referrer, a `?twa=1` start parameter) would have to be
/// persisted, and a TWA shares Chrome's cookie jar, so any persisted marker
/// would also hide billing when the same person opens the website in their
/// browser.
pub fn is_native_host(config: &Config, headers: &HeaderMap) -> bool {
    let Some(native) = config.native_app.as_ref() else {
        return false;
    };

    // Envoy preserves `Host`, but prefer `X-Forwarded-Host` when a proxy has
    // rewritten it.
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // `X-Forwarded-Host` may carry a client-to-proxy chain; the first entry is
    // the host the client actually asked for.
    let host = host.split(',').next().unwrap_or_default().trim();

    // Strip the port, taking care not to mangle a bracketed IPv6 literal.
    let host = match host.rfind(':') {
        Some(i) if !host.ends_with(']') => &host[..i],
        _ => host,
    };

    !native.host.is_empty() && host.eq_ignore_ascii_case(&native.host)
}

/// The User-Agent to persist for a new session.
///
/// The marker has to live inside the stored string because the per-device
/// dedup runs in an evento projection handler, which sees only the event and
/// never the request headers. Without it, the same Chrome signing into both
/// hosts would produce one key and each login would evict the other session.
pub fn session_ua(raw: &str, is_native: bool) -> String {
    if is_native {
        format!("{raw} {NATIVE_UA_MARKER}")
    } else {
        raw.to_owned()
    }
}

/// Route guard: rejects requests served on the native-app host.
///
/// Google Play requires digital purchases to go through Play Billing, so the
/// Stripe checkout and everything that steers toward it must not be reachable
/// inside the app. Hiding the UI is not enough on its own — a reviewer can
/// reach a route directly — so purchase handlers prove they are unreachable
/// from the app in their signature, the same way `RequireChef` proves a role.
///
/// Rejects with 404 rather than 403 on purpose: a 403 advertises that the
/// feature exists and is being withheld, which is exactly the conclusion a
/// policy reviewer should not be invited to draw.
pub struct DenyNativeApp;

impl FromRequestParts<crate::AppState> for DenyNativeApp {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        if !is_native_host(&state.config, &parts.headers) {
            return Ok(DenyNativeApp);
        }

        let template = Template::from_request_parts(parts, state)
            .await
            .expect("Infallible");

        Err((StatusCode::NOT_FOUND, template.render(NotFoundTemplate)).into_response())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_host(host: &str) -> Config {
        let mut config = Config::load(Some("../../config/default.toml".to_owned()))
            .expect("default config should load");

        config.native_app = Some(crate::config::NativeAppConfig {
            host: host.to_owned(),
            package_name: "app.imkitchen.android".to_owned(),
            sha256_cert_fingerprints: vec![],
        });

        config
    }

    fn headers(name: &str, value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
        headers
    }

    #[test]
    fn matches_the_configured_host() {
        let config = config_with_host("app.imkitchen.app");

        assert!(is_native_host(
            &config,
            &headers("host", "app.imkitchen.app")
        ));
    }

    /// The whole design rests on this: the website must never be mistaken for
    /// the app, or billing silently disappears for paying web users.
    #[test]
    fn does_not_match_the_canonical_host() {
        let config = config_with_host("app.imkitchen.app");

        assert!(!is_native_host(&config, &headers("host", "imkitchen.app")));
    }

    #[test]
    fn ignores_case_and_port() {
        let config = config_with_host("app.imkitchen.app");

        assert!(is_native_host(
            &config,
            &headers("host", "App.ImKitchen.App:3000")
        ));
    }

    #[test]
    fn prefers_the_forwarded_host() {
        let config = config_with_host("app.imkitchen.app");
        let mut h = headers("host", "imkitchen.svc.cluster.local");
        h.insert(
            header::HeaderName::from_static("x-forwarded-host"),
            "app.imkitchen.app".parse().unwrap(),
        );

        assert!(is_native_host(&config, &h));
    }

    #[test]
    fn uses_the_first_entry_of_a_forwarded_chain() {
        let config = config_with_host("app.imkitchen.app");

        assert!(is_native_host(
            &config,
            &headers("x-forwarded-host", "app.imkitchen.app, proxy.internal")
        ));
    }

    /// Section absent = no native app, so nothing about the web app changes.
    #[test]
    fn is_false_when_unconfigured() {
        let mut config = config_with_host("app.imkitchen.app");
        config.native_app = None;

        assert!(!is_native_host(
            &config,
            &headers("host", "app.imkitchen.app")
        ));
    }

    #[test]
    fn is_false_without_a_host_header() {
        let config = config_with_host("app.imkitchen.app");

        assert!(!is_native_host(&config, &HeaderMap::new()));
    }

    #[test]
    fn session_ua_marks_only_native_sessions() {
        assert_eq!(session_ua("Mozilla/5.0", false), "Mozilla/5.0");
        assert_eq!(session_ua("Mozilla/5.0", true), "Mozilla/5.0 imkitchen-twa");
    }

    /// The marker is what stops the app and browser sessions in one Chrome
    /// from evicting each other.
    #[test]
    fn session_ua_separates_app_and_browser_keys() {
        use imkitchen_types::user_agent::stable_ua_key;

        let raw = "Mozilla/5.0 (Linux; Android 10; K) Chrome/141.0.0.0 Mobile Safari/537.36";

        assert_ne!(
            stable_ua_key(&session_ua(raw, true)),
            stable_ua_key(&session_ua(raw, false))
        );
    }
}
