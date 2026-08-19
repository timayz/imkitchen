use std::io::Write;
use std::time::Duration;

use axum::body::Bytes;
use imkitchen_web_shared::{AppState, SitemapPayload};

#[derive(askama::Template)]
#[template(path = "sitemap.xml")]
pub struct SitemapTemplate {
    pub base_url: String,
    pub recipe_slugs: Vec<String>,
    pub cook_names: Vec<String>,
}

/// Upper bound on sitemap staleness. Near-real-time would buy nothing:
/// clients already cache the response for a day via Cache-Control.
const REBUILD_INTERVAL: Duration = Duration::from_secs(15 * 60);

/// Query the read model, render the sitemap and swap the pre-compressed
/// payload into [`AppState`]. Runs off the request path only.
pub async fn rebuild(app: &AppState) -> anyhow::Result<()> {
    let (recipe_slugs, cook_names) = tokio::try_join!(
        app.core.recipe.list_shared_slugs(),
        app.core.recipe.list_shared_cook_names()
    )?;

    let xml = askama::Template::render(&SitemapTemplate {
        base_url: app.config.server.url.trim_end_matches('/').to_owned(),
        recipe_slugs,
        cook_names,
    })?;

    // Compressing a multi-MB body is CPU-bound; keep it off the async workers.
    let payload = tokio::task::spawn_blocking(move || build_payload(xml.into())).await??;
    app.sitemap.store(payload);

    Ok(())
}

fn build_payload(identity: Bytes) -> anyhow::Result<SitemapPayload> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(6));
    encoder.write_all(&identity)?;
    let gzip = Bytes::from(encoder.finish()?);

    let mut brotli = Vec::new();
    // quality 5 ≈ per-request middleware ratios at a fraction of the cost;
    // affordable here because it runs once per rebuild, not per request.
    let mut writer = brotli::CompressorWriter::new(&mut brotli, 4096, 5, 22);
    writer.write_all(&identity)?;
    drop(writer);

    Ok(SitemapPayload {
        identity,
        gzip,
        brotli: Bytes::from(brotli),
    })
}

/// Background loop: rebuilds on a fixed schedule. Keeps serving the previous
/// payload if a rebuild fails.
pub fn spawn(app: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(REBUILD_INTERVAL);
        interval.tick().await; // first tick fires immediately; startup already built

        loop {
            interval.tick().await;

            if let Err(err) = rebuild(&app).await {
                tracing::error!(?err, "sitemap rebuild failed; serving previous payload");
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use askama::Template;

    use super::{SitemapTemplate, build_payload};

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

    #[test]
    fn payload_variants_round_trip() {
        let xml = "<urlset>".repeat(1000);
        let payload = build_payload(xml.clone().into()).unwrap();

        assert_eq!(payload.identity, xml.as_bytes());

        let mut gunzipped = String::new();
        flate2::read::GzDecoder::new(&payload.gzip[..])
            .read_to_string(&mut gunzipped)
            .unwrap();
        assert_eq!(gunzipped, xml);

        let mut unbrotlied = String::new();
        brotli::Decompressor::new(&payload.brotli[..], 4096)
            .read_to_string(&mut unbrotlied)
            .unwrap();
        assert_eq!(unbrotlied, xml);
    }
}
