use evento::Executor;

use crate::types::Visited;

pub struct RecordVisitInput {
    pub path: String,
    pub user_agent: String,
    pub timezone: String,
    /// The landing page's `document.referrer`, when the browser exposes one.
    pub referrer: Option<String>,
}

impl<E: Executor + Clone> crate::Module<E> {
    /// Records a landing-page visit. Returns `Ok(None)` for bots — nothing is
    /// committed. Never validates: the beacon is fire-and-forget, so bad input
    /// degrades to "other"/"ZZ" buckets instead of erroring.
    pub async fn record_visit(&self, input: RecordVisitInput) -> anyhow::Result<Option<String>> {
        let ua = crate::ua::classify(&input.user_agent);
        if ua.is_bot {
            return Ok(None);
        }

        let country = crate::tz_country::country_for_tz(&input.timezone)
            .unwrap_or("ZZ")
            .to_owned();

        let id = evento::create()
            .event(&Visited {
                path: normalize_path(&input.path),
                device: truncate(ua.device, 50),
                browser: truncate(ua.browser, 50),
                os: truncate(ua.os, 50),
                country,
                timezone: truncate(input.timezone, 64),
                referrer: normalize_referrer(input.referrer.as_deref()),
            })
            .commit(&self.executor)
            .await?;

        Ok(Some(id))
    }
}

/// Collapses the reported path to a closed set so rollup cardinality stays
/// bounded on an unauthenticated endpoint.
fn normalize_path(path: &str) -> String {
    let path = path.split('?').next().unwrap_or("/");
    match path {
        "/" => "/",
        "/about" => "/about",
        _ => "other",
    }
    .to_owned()
}

/// Reduces a raw referrer URL to its lowercased host without a `www.` prefix
/// ("https://www.Google.com/search?q=x" -> "google.com"). Anything absent or
/// unparseable becomes "direct".
fn normalize_referrer(referrer: Option<&str>) -> String {
    let host = referrer
        .filter(|value| !value.trim().is_empty())
        .and_then(|value| url::Url::parse(value).ok())
        .and_then(|url| url.host_str().map(str::to_lowercase));

    match host {
        Some(host) => {
            let host = host.strip_prefix("www.").unwrap_or(&host);
            truncate(host.to_owned(), 100)
        }
        None => "direct".to_owned(),
    }
}

fn truncate(value: String, max: usize) -> String {
    if value.len() <= max {
        value
    } else {
        value.chars().take(max).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_referrers() {
        assert_eq!(
            normalize_referrer(Some("https://www.Google.com/search?q=x")),
            "google.com"
        );
        assert_eq!(normalize_referrer(Some("https://t.co/abc")), "t.co");
        assert_eq!(normalize_referrer(Some("not a url")), "direct");
        assert_eq!(normalize_referrer(Some("")), "direct");
        assert_eq!(normalize_referrer(None), "direct");
    }

    #[test]
    fn normalizes_paths() {
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path("/?utm_source=x"), "/");
        assert_eq!(normalize_path("/about"), "/about");
        assert_eq!(normalize_path("/anything/else"), "other");
        assert_eq!(normalize_path(""), "other");
    }
}
