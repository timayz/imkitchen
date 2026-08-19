use axum::{extract::State, http::header, response::IntoResponse};

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

#[derive(askama::Template)]
#[template(path = "sitemap.xml")]
pub struct SitemapTemplate {
    pub base_url: String,
    pub recipe_slugs: Vec<String>,
    pub cook_names: Vec<String>,
}

pub async fn sitemap(template: Template, State(app): State<AppState>) -> impl IntoResponse {
    let recipe_slugs =
        imkitchen_web_shared::try_page_response!(app.core.recipe.list_shared_slugs(), template);
    let cook_names = imkitchen_web_shared::try_page_response!(
        app.core.recipe.list_shared_cook_names(),
        template
    );

    (
        [
            (
                header::CONTENT_TYPE.as_str(),
                "application/xml; charset=utf-8",
            ),
            (header::CACHE_CONTROL.as_str(), "public, max-age=86400"),
        ],
        template.render(SitemapTemplate {
            base_url: app.config.server.url.trim_end_matches('/').to_owned(),
            recipe_slugs,
            cook_names,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use askama::Template;

    use super::SitemapTemplate;

    #[test]
    fn sitemap_renders_config_urls_and_dynamic_entries() {
        let xml = SitemapTemplate {
            base_url: "https://example.test".into(),
            recipe_slugs: vec!["arroz-con-pollo".into()],
            cook_names: vec!["alice".into()],
        }
        .render()
        .unwrap();

        assert!(xml.contains("<loc>https://example.test/</loc>"));
        assert!(xml.contains("<loc>https://example.test/legal</loc>"));
        assert!(xml.contains("<loc>https://example.test/demo</loc>"));
        assert!(xml.contains("<loc>https://example.test/r/arroz-con-pollo</loc>"));
        assert!(xml.contains("<loc>https://example.test/cooks/alice</loc>"));
        assert!(!xml.contains("imkitchen.app"));
        assert!(!xml.contains("/login"));
    }
}
