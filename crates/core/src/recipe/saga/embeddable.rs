//! Background subscription that determines whether a recipe's `origin` site
//! permits iframe embedding, caching the verdict per-domain in `origin_framing`.
//!
//! The check is a network side-effect — an HTTP GET reading `X-Frame-Options` /
//! CSP `frame-ancestors` — so it runs here, off the request path, instead of
//! lazily when the cooking screen is rendered. Because the cache is keyed by
//! domain and framing policy is effectively site-wide, each domain is fetched at
//! most once; recipes sharing a domain reuse the cached verdict, and a fresh
//! deploy backfills every existing recipe's domain via the subscription replay.
//! Request handlers ([`crate::recipe::Module::is_origin_embeddable`]) then only
//! read the cache.

use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_types::recipe::{BasicInformationChanged, Imported};
use sqlx::SqlitePool;

use crate::recipe::query::embeddable::{check_embeddable, domain_of, is_domain_cached, store_verdict};

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-saga-embeddable")
        .handler(handle_imported())
        .handler(handle_basic_information_changed())
}

/// Checks `origin`'s framing headers once per domain and caches the verdict.
async fn ensure_checked(
    read_db: &SqlitePool,
    write_db: &SqlitePool,
    origin: Option<&str>,
) -> anyhow::Result<()> {
    let Some(origin) = origin else {
        return Ok(());
    };
    let Some(domain) = domain_of(origin) else {
        return Ok(());
    };

    // One network check per domain — skip if a verdict already exists.
    if is_domain_cached(read_db, &domain).await? {
        return Ok(());
    }

    let embeddable = check_embeddable(origin).await;
    store_verdict(write_db, &domain, embeddable).await
}

#[evento::subscription]
async fn handle_imported<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Imported>,
) -> anyhow::Result<()> {
    let (read_db, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
    ensure_checked(&read_db, &write_db, event.data.origin.as_deref()).await
}

#[evento::subscription]
async fn handle_basic_information_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<BasicInformationChanged>,
) -> anyhow::Result<()> {
    let (read_db, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
    ensure_checked(&read_db, &write_db, event.data.origin.as_deref()).await
}
