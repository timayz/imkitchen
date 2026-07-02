use std::time::Duration;

use evento::Executor;
use imkitchen_db::origin_framing::OriginFraming;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

impl<E: Executor> crate::recipe::Module<E> {
    /// Returns whether the site behind `origin` permits being embedded in an
    /// iframe, from the per-domain `origin_framing` cache.
    ///
    /// This is a pure read — the actual header check is performed off the request
    /// path by the background `recipe-saga-embeddable` subscription when a recipe
    /// is imported or its origin changes (see [`crate::recipe::saga::embeddable`]).
    /// A domain that has not been checked yet, or whose check failed, reads as
    /// `false`, so the caller falls back to the native step UI / external link.
    pub async fn is_origin_embeddable(&self, origin: &str) -> anyhow::Result<bool> {
        let Some(domain) = domain_of(origin) else {
            return Ok(false);
        };

        let statement = Query::select()
            .column(OriginFraming::Embeddable)
            .from(OriginFraming::Table)
            .and_where(Expr::col(OriginFraming::Domain).eq(domain))
            .to_owned();
        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, (bool,), _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_optional(&self.read_db)
                .await?
                .map(|(embeddable,)| embeddable)
                .unwrap_or(false),
        )
    }
}

/// Lower-cased host of a URL, used as the per-domain cache key.
pub(crate) fn domain_of(origin: &str) -> Option<String> {
    reqwest::Url::parse(origin)
        .ok()?
        .host_str()
        .map(str::to_ascii_lowercase)
}

/// Whether the `origin_framing` cache already holds a verdict for `domain`.
pub(crate) async fn is_domain_cached(read_db: &SqlitePool, domain: &str) -> anyhow::Result<bool> {
    let statement = Query::select()
        .column(OriginFraming::Domain)
        .from(OriginFraming::Table)
        .and_where(Expr::col(OriginFraming::Domain).eq(domain.to_owned()))
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(
        sqlx::query_as_with::<_, (String,), _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_optional(read_db)
            .await?
            .is_some(),
    )
}

/// Persists the framing verdict for `domain`, stamped with the current time.
pub(crate) async fn store_verdict(
    write_db: &SqlitePool,
    domain: &str,
    embeddable: bool,
) -> anyhow::Result<()> {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let statement = Query::insert()
        .into_table(OriginFraming::Table)
        .columns([
            OriginFraming::Domain,
            OriginFraming::Embeddable,
            OriginFraming::CheckedAt,
        ])
        .values_panic([domain.to_owned().into(), embeddable.into(), now.into()])
        .on_conflict(
            OnConflict::column(OriginFraming::Domain)
                .update_columns([OriginFraming::Embeddable, OriginFraming::CheckedAt])
                .to_owned(),
        )
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(write_db)
        .await?;

    Ok(())
}

/// Fetches `origin` and inspects its framing headers. Returns `false` on any
/// network/HTTP failure so blocked or unreachable sites degrade to the native
/// fallback.
pub(crate) async fn check_embeddable(origin: &str) -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .user_agent("Mozilla/5.0 (compatible; imkitchen)")
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };

    let response = match client.get(origin).send().await {
        Ok(response) => response,
        Err(_) => return false,
    };

    let headers = response.headers();

    // X-Frame-Options: DENY / SAMEORIGIN both forbid embedding on our origin.
    if let Some(xfo) = headers
        .get("x-frame-options")
        .and_then(|value| value.to_str().ok())
    {
        let xfo = xfo.to_ascii_lowercase();
        if xfo.contains("deny") || xfo.contains("sameorigin") {
            return false;
        }
    }

    // CSP frame-ancestors: only a wildcard source permits arbitrary embedders.
    // 'none', 'self', or a specific host list all block us.
    if let Some(csp) = headers
        .get("content-security-policy")
        .and_then(|value| value.to_str().ok())
    {
        let csp = csp.to_ascii_lowercase();
        if let Some(idx) = csp.find("frame-ancestors") {
            let directive = csp[idx + "frame-ancestors".len()..]
                .split(';')
                .next()
                .unwrap_or("");
            if !directive.contains('*') {
                return false;
            }
        }
    }

    true
}
