use std::{path::PathBuf, str::FromStr};

use evento::{
    Sqlite,
    migrator::{Migrate, Plan},
};
use imkitchen_audience::daily_stat::BreakdownDim;
use imkitchen_audience::{Module, RecordVisitInput, State};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use temp_dir::TempDir;

const CHROME_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const IPHONE_UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
const GOOGLEBOT_UA: &str =
    "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)";

async fn setup_test_module(path: PathBuf) -> anyhow::Result<Module<Sqlite>> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    let mut conn = pool.acquire().await?;
    imkitchen_audience::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;

    Ok(Module::new(State {
        executor: pool.clone().into(),
        read_db: pool.clone(),
        write_db: pool,
    }))
}

async fn run_daily_stat_subscription(module: &Module<Sqlite>) -> anyhow::Result<()> {
    imkitchen_audience::daily_stat::subscription()
        .data(module.write_db.clone())
        .no_retry()
        .run_once(&module.executor)
        .await?;
    Ok(())
}

fn visit(path: &str, user_agent: &str, timezone: &str) -> RecordVisitInput {
    RecordVisitInput {
        path: path.to_owned(),
        user_agent: user_agent.to_owned(),
        timezone: timezone.to_owned(),
        referrer: None,
    }
}

fn visit_from(referrer: &str, path: &str, user_agent: &str, timezone: &str) -> RecordVisitInput {
    RecordVisitInput {
        referrer: Some(referrer.to_owned()),
        ..visit(path, user_agent, timezone)
    }
}

#[tokio::test]
async fn test_record_visit() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let module = setup_test_module(dir.child("audience.sqlite3")).await?;

    let id = module
        .record_visit(visit("/", CHROME_UA, "Europe/Paris"))
        .await?;
    assert!(id.is_some());

    Ok(())
}

#[tokio::test]
async fn test_bots_are_not_recorded() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let module = setup_test_module(dir.child("audience.sqlite3")).await?;

    let id = module
        .record_visit(visit("/", GOOGLEBOT_UA, "Europe/Paris"))
        .await?;
    assert!(id.is_none());

    run_daily_stat_subscription(&module).await?;
    assert_eq!(module.total_since("0000-00-00").await?, 0);

    Ok(())
}

#[tokio::test]
async fn test_daily_stat_rollup() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let module = setup_test_module(dir.child("audience.sqlite3")).await?;

    // Two identical-dimension visits plus one from another country/device.
    module
        .record_visit(visit_from(
            "https://www.google.com/search?q=imkitchen",
            "/?utm_source=x",
            CHROME_UA,
            "Europe/Paris",
        ))
        .await?;
    module
        .record_visit(visit_from(
            "https://google.com/",
            "/",
            CHROME_UA,
            "Europe/Paris",
        ))
        .await?;
    module
        .record_visit(visit("/about", IPHONE_UA, "America/New_York"))
        .await?;

    run_daily_stat_subscription(&module).await?;

    assert_eq!(module.total_since("0000-00-00").await?, 3);

    let per_day = module.per_day("0000-00-00").await?;
    assert_eq!(per_day.len(), 1);
    assert_eq!(per_day[0].total, 3);

    let countries = module
        .breakdown(BreakdownDim::Country, "0000-00-00", 10)
        .await?;
    assert_eq!(countries.len(), 2);
    assert_eq!((countries[0].label.as_str(), countries[0].total), ("FR", 2));
    assert_eq!((countries[1].label.as_str(), countries[1].total), ("US", 1));

    let devices = module
        .breakdown(BreakdownDim::Device, "0000-00-00", 10)
        .await?;
    assert_eq!((devices[0].label.as_str(), devices[0].total), ("pc", 2));
    assert_eq!(
        (devices[1].label.as_str(), devices[1].total),
        ("smartphone", 1)
    );

    let paths = module
        .breakdown(BreakdownDim::Path, "0000-00-00", 10)
        .await?;
    assert_eq!((paths[0].label.as_str(), paths[0].total), ("/", 2));
    assert_eq!((paths[1].label.as_str(), paths[1].total), ("/about", 1));

    let referrers = module
        .breakdown(BreakdownDim::Referrer, "0000-00-00", 10)
        .await?;
    assert_eq!(
        (referrers[0].label.as_str(), referrers[0].total),
        ("google.com", 2)
    );
    assert_eq!(
        (referrers[1].label.as_str(), referrers[1].total),
        ("direct", 1)
    );

    // All events share the same commit second, so force distinct last-seen
    // times to make the recency ordering observable.
    sqlx::query(
        "update audience_daily_stat set updated_at = updated_at + 60 where referrer = 'direct'",
    )
    .execute(&module.write_db)
    .await?;
    let recent = module.recent_referrers("0000-00-00", 10).await?;
    assert_eq!((recent[0].label.as_str(), recent[0].total), ("direct", 1));
    assert_eq!(
        (recent[1].label.as_str(), recent[1].total),
        ("google.com", 2)
    );

    // Idempotent rollup: draining again must not double-count.
    run_daily_stat_subscription(&module).await?;
    assert_eq!(module.total_since("0000-00-00").await?, 3);

    Ok(())
}

#[tokio::test]
async fn test_unknown_timezone_becomes_zz() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let module = setup_test_module(dir.child("audience.sqlite3")).await?;

    module
        .record_visit(visit("/", CHROME_UA, "Not/AZone"))
        .await?;
    run_daily_stat_subscription(&module).await?;

    let countries = module
        .breakdown(BreakdownDim::Country, "0000-00-00", 10)
        .await?;
    assert_eq!((countries[0].label.as_str(), countries[0].total), ("ZZ", 1));

    Ok(())
}
