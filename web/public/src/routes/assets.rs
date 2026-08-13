use axum::{
    extract::State,
    http::{StatusCode, header},
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

/// Digital asset links proving the Android TWA (Play Store app) may open
/// imkitchen URLs without browser chrome. 404 until `[android]` is configured.
pub async fn assetlinks(State(app): State<AppState>) -> impl IntoResponse {
    let Some(ref android) = app.config.android else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let statements = serde_json::json!([{
        "relation": ["delegate_permission/common.handle_all_urls"],
        "target": {
            "namespace": "android_app",
            "package_name": android.package_name,
            "sha256_cert_fingerprints": android.sha256_cert_fingerprints,
        }
    }]);

    (
        [
            (
                header::CONTENT_TYPE.as_str(),
                "application/json; charset=utf-8",
            ),
            (header::CACHE_CONTROL.as_str(), "public, max-age=3600"),
        ],
        statements.to_string(),
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

#[derive(askama::Template)]
#[template(path = "sitemap.xml")]
pub struct SitemapTemplate;

pub async fn sitemap(template: Template) -> impl IntoResponse {
    (
        [
            (
                header::CONTENT_TYPE.as_str(),
                "application/xml; charset=utf-8",
            ),
            (header::CACHE_CONTROL.as_str(), "public, max-age=86400"),
        ],
        template.render(SitemapTemplate),
    )
}
