use evento::Executor;

use crate::types::Visited;

pub struct RecordVisitInput {
    pub path: String,
    pub user_agent: String,
    pub timezone: String,
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
    fn normalizes_paths() {
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path("/?utm_source=x"), "/");
        assert_eq!(normalize_path("/about"), "/about");
        assert_eq!(normalize_path("/anything/else"), "other");
        assert_eq!(normalize_path(""), "other");
    }
}
