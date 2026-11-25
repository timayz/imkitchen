//! The store: one Turso file per node.
//!
//! It holds the events, the snapshots, the subscription cursors, the cluster
//! journal and the api's own tables, and is opened for several writers at
//! once (Turso's MVCC, `BEGIN CONCURRENT`): the node applies events on one
//! connection, the journal writes on another, and the api's tables are
//! written through [`evento::Turso::transaction`]. Two writes only collide
//! when they change the same rows, and the one that lost is run again.
//!
//! The schema is created by migrations alone ([`migrate`], the `migrate`
//! command); `serve` only opens what they left ([`open`]).

use std::path::Path;

use anyhow::{Context, Result};
use evento::Turso;
use evento::turso::migrate::Migrations;
use evento::turso::{TursoJournal, turso::Connection};

pub type Store = Turso;

/// The api's own tables, next to evento's in the same file. Append only: a
/// migration that was released is never edited.
pub const MIGRATIONS: Migrations<'static> = Migrations::new("imkitchen-api", &[]);

/// Every list of migrations the store's schema is made of.
const SCHEMA: [&Migrations<'static>; 3] = [
    &evento::turso::MIGRATIONS,
    &evento::turso::CLUSTER_MIGRATIONS,
    &MIGRATIONS,
];

/// Creates the store at `path` if needed, file and directory, and applies
/// the pending migrations: the events', the cluster journal's, then the
/// api's own.
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
    // The node builds its journal over the same database later.
    TursoJournal::new(store.database().clone())
        .await
        .with_context(|| format!("cannot migrate the cluster journal in {}", path.display()))?;
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
        "there is no store at {}: run `imkitchen-api migrate` first",
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
        "the store at {} has migrations to apply ({}): run `imkitchen-api migrate` first",
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
