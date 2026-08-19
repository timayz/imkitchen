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
pub struct ManifestTemplate;

pub async fn manifest(template: Template) -> impl IntoResponse {
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
        template.render(ManifestTemplate),
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
