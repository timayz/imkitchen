//! The store of www: one Turso file of its own.
//!
//! www owns no aggregate and writes no event. Its file holds what it derives
//! from the api's events: the read models its pages are read from ([`Db`])
//! and, in evento's tables next to them, the cursor of each subscription
//! that fills them. Nothing in it is written anywhere else, so a file that
//! is lost is built again from the api, and each replica of www keeps its
//! own.
//!
//! It is opened for several writers at once (Turso's MVCC): a subscription
//! writes its read model and its cursor while the pages read.
//!
//! The schema is created by migrations alone ([`migrate`], the `migrate`
//! command); `serve` only opens what they left ([`open`]).

use std::path::Path;

use anyhow::{Context, Result};
use evento::Turso;
use evento::turso::migrate::{Migration, Migrations};
use evento::turso::turso::Connection;
/// The read models' side of the store: the tables next to evento's, read by
/// the pages and written by the subscriptions.
pub use timada_core::db::Db;

pub type Store = Turso;

/// The tables of www, next to evento's in the same file. Append only: a
/// migration that was released is never edited.
pub const MIGRATIONS: Migrations<'static> = Migrations::new(
    "imkitchen-www",
    &[Migration::up(
        "0001_recipes",
        // One row per recipe that is not deleted, and nothing of what it
        // says: how many there are is all www shows of them.
        "CREATE TABLE recipes (id TEXT NOT NULL PRIMARY KEY);",
    )],
);

/// Every list of migrations the store's schema is made of.
const SCHEMA: [&Migrations<'static>; 2] = [&evento::turso::MIGRATIONS, &MIGRATIONS];

/// Creates the store at `path` if needed, file and directory, and applies
/// the pending migrations: evento's, then www's own.
pub async fn migrate(path: &Path) -> Result<Store> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    let store = Turso::open_concurrent(path)
        .await
        .with_context(|| format!("cannot open {}", path.display()))?;
    let mut conn = store.connect()?;
    MIGRATIONS
        .to_latest(&mut conn)
        .await
        .map_err(|(applied, err)| {
            anyhow::anyhow!(
                "cannot migrate {} (after {applied} migrations): {err}",
                path.display()
            )
        })?;
    tracing::info!(path = %path.display(), "store ready");
    Ok(store)
}

/// Opens the store at `path` as [`migrate`] left it and changes nothing in
/// its schema: a store that is not there, or one with migrations still to
/// apply, is refused. When the schema changes is the operator's to say.
pub async fn open(path: &Path) -> Result<Store> {
    anyhow::ensure!(
        path.is_file(),
        "there is no store at {}: run `imkitchen-www migrate` first",
        path.display()
    );
    let database = Turso::builder(
        path.to_str()
            .with_context(|| format!("{} is not valid UTF-8", path.display()))?,
    )
    .build()
    .await
    .with_context(|| format!("cannot open {}", path.display()))?;
    let conn = database
        .connect()
        .with_context(|| format!("cannot open {}", path.display()))?;
    let pending = pending_migrations(&conn)
        .await
        .with_context(|| format!("cannot read the migrations of {}", path.display()))?;
    drop(conn);
    anyhow::ensure!(
        pending.is_empty(),
        "the store at {} has migrations to apply ({}): run `imkitchen-www migrate` first",
        path.display(),
        pending.join(", ")
    );
    let store = Turso::from_database(database)
        .await
        .with_context(|| format!("cannot open {}", path.display()))?;
    tracing::info!(path = %path.display(), "store ready");
    Ok(store)
}

/// The lists with migrations the store has not recorded, each as "name: how
/// many". A store that records more than this version knows was migrated by
/// a newer one, which this one must not run on.
async fn pending_migrations(conn: &Connection) -> Result<Vec<String>> {
    let mut pending = Vec::new();
    for migrations in SCHEMA {
        let app = migrations.app();
        let applied = migrations
            .applied(conn)
            .await
            .with_context(|| format!("{app}: cannot read what was applied"))?
            .len();
        anyhow::ensure!(
            applied <= migrations.len(),
            "{app}: the store records {applied} migrations and this version knows {}: \
             it was migrated by a newer version",
            migrations.len()
        );
        if applied < migrations.len() {
            pending.push(format!("{app}: {}", migrations.len() - applied));
        }
    }
    Ok(pending)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_store_is_opened_as_migrate_left_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("www.db");

        let Err(err) = open(&path).await else {
            panic!("opened a missing store");
        };
        assert!(err.to_string().contains("migrate"), "{err}");

        drop(migrate(&path).await.unwrap());
        // Again: nothing is pending.
        drop(migrate(&path).await.unwrap());
        let db = Db::from(open(&path).await.unwrap());
        let recipes: i64 = db
            .fetch_scalar("SELECT count(*) FROM recipes", ())
            .await
            .unwrap();
        assert_eq!(recipes, 0);
    }
}
