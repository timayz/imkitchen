use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};

use imkitchen_web_shared::{
    AppState,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "manifest.json")]
pub struct ManifestTemplate {
    pub play_package_name: Option<String>,
}

pub async fn manifest(template: Template, State(app): State<AppState>) -> impl IntoResponse {
    (
        [
            (
                header::CONTENT_TYPE.as_str(),
                "application/json; charset=utf-8",
            ),
            (
                header::CACHE_CONTROL.as_str(),
                "no-cache, no-store, must-revalidate",
            ),
        ],
        template.render(ManifestTemplate {
            play_package_name: app
                .config
                .native_app
                .as_ref()
                .map(|native| native.package_name.to_owned()),
        }),
    )
}

#[derive(askama::Template)]
#[template(path = "sw.js")]
pub struct ServiceWorkerTemplate;

/// Serve service worker at /sw.js (must be at root for scope: '/')
pub async fn service_worker(template: Template) -> impl IntoResponse {
    (
        [
            (
                header::CONTENT_TYPE.as_str(),
                "application/javascript; charset=utf-8",
            ),
            (
                header::CACHE_CONTROL.as_str(),
                "no-cache, no-store, must-revalidate",
            ),
            ("Service-Worker-Allowed", "/"),
        ],
        template.render(ServiceWorkerTemplate),
    )
}

#[derive(askama::Template)]
#[template(path = "assetlinks.json")]
pub struct AssetLinksTemplate {
    pub package_name: String,
    pub fingerprints: Vec<String>,
}

/// Digital Asset Links statement, proving this site and the Android app
/// belong to the same owner. Without it the TWA falls back to showing a
/// browser URL bar.
///
/// Must stay unauthenticated, served over HTTPS at exactly this path, and
/// never redirected — Google's verifier follows none of that.
///
/// Driven by config rather than baked into the template because the app
/// signing fingerprint only exists *after* the first upload to Play, so
/// adding it has to be a redeploy rather than an image rebuild. Cached
/// briefly for the same reason; setting Cache-Control here also stops
/// `cache_control_middleware` stamping its no-store default.
pub async fn assetlinks(template: Template, State(app): State<AppState>) -> impl IntoResponse {
    let Some(native) = app.config.native_app.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    (
        [
            (
                header::CONTENT_TYPE.as_str(),
                "application/json; charset=utf-8",
            ),
            (header::CACHE_CONTROL.as_str(), "public, max-age=300"),
        ],
        template.render(AssetLinksTemplate {
            package_name: native.package_name.to_owned(),
            fingerprints: native.sha256_cert_fingerprints.to_owned(),
        }),
    )
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "robots.txt")]
pub struct RobotsTemplate;

pub async fn robots(template: Template) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE.as_str(), "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL.as_str(), "public, max-age=86400"),
        ],
        template.render(RobotsTemplate),
    )
}

/// Serve the pre-built sitemap maintained by `crate::sitemap`. Pure in-memory
/// lookup: no queries, no rendering, no per-request compression — setting
/// Content-Encoding ourselves makes the CompressionLayer skip the response.
pub async fn sitemap(State(app): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let payload = app.sitemap.load();
    if payload.identity.is_empty() {
        // Only reachable if the startup build failed; the rebuild task retries.
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::RETRY_AFTER.as_str(), "30")],
            "",
        )
            .into_response();
    }

    let accept_encoding = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // Token-presence negotiation (q-values ignored): real crawlers all send
    // plain "gzip, deflate, br"-style headers.
    let (body, content_encoding) = if accept_encoding.contains("br") {
        (payload.brotli.clone(), Some("br"))
    } else if accept_encoding.contains("gzip") {
        (payload.gzip.clone(), Some("gzip"))
    } else {
        (payload.identity.clone(), None)
    };

    let mut response = (
        [
            (
                header::CONTENT_TYPE.as_str(),
                "application/xml; charset=utf-8",
            ),
            (header::CACHE_CONTROL.as_str(), "public, max-age=86400"),
            (header::VARY.as_str(), "accept-encoding"),
        ],
        body,
    )
        .into_response();

    if let Some(encoding) = content_encoding {
        response
            .headers_mut()
            .insert(header::CONTENT_ENCODING, encoding.parse().unwrap());
    }

    response
}
