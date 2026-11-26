//! What admin knows of the imports of recipes: each try, the recipe it
//! gave or why it gave none, and the sites they are made from.
//!
//! admin keeps one row per try, which is one event: written once however
//! often the event is delivered (the subscription is at-least-once). A try
//! outlives the recipe it gave, should its owner delete it.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::recipe_import::{Failed, Imported};
use reqwest::Url;
use timada_core::db::FromRow;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-admin/recipe-imports";

/// Follows the imports into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_imported())
        .handler(on_failed())
}

/// The site of the page at `url`, what the sources are counted by: its
/// host, lowercased, without the `www.` most sites answer with or without.
/// Empty when `url` names none.
pub fn host(url: &str) -> String {
    let host = Url::parse(url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_lowercase))
        .unwrap_or_default();
    host.strip_prefix("www.").unwrap_or(&host).to_owned()
}

/// One try, as its event says it went.
struct Try<'a> {
    id: &'a str,
    user_id: &'a str,
    url: &'a str,
    page_sent: bool,
    recipe_id: Option<&'a str>,
    reason: Option<&'a str>,
    at: u64,
}

async fn insert(db: &Db, tried: Try<'_>) -> anyhow::Result<()> {
    db.execute(
        "INSERT INTO recipe_imports (id, user_id, url, host, page_sent, recipe_id, reason, at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT (id) DO NOTHING",
        (
            tried.id,
            tried.user_id,
            tried.url,
            host(tried.url),
            tried.page_sent,
            tried.recipe_id,
            tried.reason,
            tried.at as i64,
        ),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_imported<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Imported>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let data = &event.data;
    insert(
        &db,
        Try {
            id: &event.aggregate_id,
            user_id: &data.user_id,
            url: &data.url,
            page_sent: data.page_sent,
            recipe_id: Some(&data.recipe_id),
            reason: None,
            at: event.timestamp,
        },
    )
    .await
}

#[evento::subscription]
async fn on_failed<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Failed>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let data = &event.data;
    insert(
        &db,
        Try {
            id: &event.aggregate_id,
            user_id: &data.user_id,
            url: &data.url,
            page_sent: data.page_sent,
            recipe_id: None,
            reason: Some(&data.reason),
            at: event.timestamp,
        },
    )
    .await
}

/// One try at importing a recipe.
#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ImportRow {
    pub id: String,
    pub user_id: String,
    /// The address of the page, as the user gave it.
    pub url: String,
    pub host: String,
    /// Whether the client sent the page in place of the api fetching it.
    pub page_sent: bool,
    /// The recipe it gave, which may be deleted since.
    pub recipe_id: Option<String>,
    /// Why it gave none, as the word the api refused it with.
    pub reason: Option<String>,
    /// Unix seconds.
    pub at: i64,
}

impl ImportRow {
    pub fn failed(&self) -> bool {
        self.recipe_id.is_none()
    }
}

/// Which tries a list holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Filter<'a> {
    /// Only those that gave no recipe.
    pub failed: bool,
    /// Only those from this site.
    pub host: Option<&'a str>,
}

const FILTERED: &str = "(?1 = 0 OR recipe_id IS NULL) AND (?2 IS NULL OR host = ?2)";

/// How many tries `filter` holds, as far as admin has followed.
pub async fn count(db: &Db, filter: Filter<'_>) -> anyhow::Result<u64> {
    let count: i64 = db
        .fetch_scalar(
            &format!("SELECT count(*) FROM recipe_imports WHERE {FILTERED}"),
            (filter.failed, filter.host),
        )
        .await?;
    Ok(count.max(0) as u64)
}

/// The tries `filter` holds, the most recent first: `limit` of them from
/// the `offset`th.
pub async fn list(
    db: &Db,
    filter: Filter<'_>,
    offset: i64,
    limit: i64,
) -> anyhow::Result<Vec<ImportRow>> {
    db.fetch_all(
        &format!(
            "SELECT id, user_id, url, host, page_sent, recipe_id, reason, at FROM recipe_imports
             WHERE {FILTERED} ORDER BY at DESC, id DESC LIMIT ?3 OFFSET ?4"
        ),
        (filter.failed, filter.host, limit, offset),
    )
    .await
}

/// The try the recipe `recipe_id` came from, when it was imported.
pub async fn of_recipe(db: &Db, recipe_id: &str) -> anyhow::Result<Option<ImportRow>> {
    db.fetch_optional(
        "SELECT id, user_id, url, host, page_sent, recipe_id, reason, at FROM recipe_imports
         WHERE recipe_id = ?1",
        (recipe_id,),
    )
    .await
}

/// A site recipes are imported from, and how its tries went.
#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct Source {
    pub host: String,
    /// The tries that gave a recipe.
    pub imported: i64,
    /// Those that gave none.
    pub failed: i64,
    /// Those the client sent the page of: the site refuses the api.
    pub page_sent: i64,
    /// Unix seconds of the last try.
    pub last_at: i64,
}

/// The sites recipes are imported from, the most tried first, `limit` at
/// most.
pub async fn sources(db: &Db, limit: i64) -> anyhow::Result<Vec<Source>> {
    db.fetch_all(
        "SELECT host, count(recipe_id) AS imported, count(*) - count(recipe_id) AS failed,
            sum(page_sent) AS page_sent, max(at) AS last_at
         FROM recipe_imports GROUP BY host
         ORDER BY count(*) DESC, host LIMIT ?1",
        (limit,),
    )
    .await
}

/// How many sites recipes are imported from.
pub async fn count_sources(db: &Db) -> anyhow::Result<u64> {
    let count: i64 = db
        .fetch_scalar("SELECT count(DISTINCT host) FROM recipe_imports", ())
        .await?;
    Ok(count.max(0) as u64)
}
