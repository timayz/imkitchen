//! The store: one Turso file per node.
//!
//! It holds the events, the snapshots, the subscription cursors, the cluster
//! journal and the api's own tables, and is opened for several writers at
//! once (Turso's MVCC, `BEGIN CONCURRENT`): the node applies events on one
//! connection, the journal writes on another, and the api's tables are
//! written through [`evento::Turso::transaction`]. Two writes only collide
//! when they change the same rows, and the one that lost is run again.
//!
//! The api's tables are read models ([`Db`]): what a list is read from,
//! derived from the events of this node's file by a subscription every node
//! runs for itself. Nothing else is kept in them, so they can be emptied and
//! filled again ([`reset_read_models`]).
//!
//! The schema is created by migrations alone ([`migrate`], the `migrate`
//! command); `serve` only opens what they left ([`open`]).

use std::path::Path;

use anyhow::{Context, Result};
use evento::Turso;
use evento::turso::migrate::{MIGRATIONS_TABLE, Migration, Migrations};
use evento::turso::{TursoJournal, turso::Connection};
/// The read models' side of the store: the tables next to the events, read
/// by the endpoints and written by the subscriptions.
pub use timada_core::db::Db;

pub type Store = Turso;

/// The api's own tables, next to evento's in the same file. Append only: a
/// migration that was released is never edited.
pub const MIGRATIONS: Migrations<'static> = Migrations::new(
    "imkitchen-api",
    &[Migration::up(
        "0001_recipes",
        // One row per recipe that is not deleted, for its owner's list.
        // `search` is what a search looks in: the title and the names of
        // the ingredients, lowercased and without accents.
        "CREATE TABLE recipes (
            id TEXT NOT NULL PRIMARY KEY,
            owner_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            prep_minutes INTEGER NOT NULL,
            cook_minutes INTEGER NOT NULL,
            servings INTEGER NOT NULL,
            search TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE INDEX recipes_listed ON recipes (owner_id, created_at DESC, id DESC);",
    )],
);

/// The tables [`MIGRATIONS`] creates: all of them read models.
const READ_MODELS: &[&str] = &["recipes"];

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

/// Empties the read models: the node does it before it copies its store
/// from a peer, and its subscriptions fill them again from the copied
/// events. The tables are dropped and created again by their migrations
/// rather than emptied row by row, which would write every row once more
/// to the log. Nothing else may be using them.
pub async fn reset_read_models(store: &Store) -> Result<()> {
    let mut conn = store.connect()?;
    for table in READ_MODELS {
        conn.execute(&format!("DROP TABLE IF EXISTS {table}"), ())
            .await
            .with_context(|| format!("cannot drop {table}"))?;
    }
    conn.execute(
        &format!("DELETE FROM {MIGRATIONS_TABLE} WHERE app = ?1"),
        [MIGRATIONS.app()],
    )
    .await
    .context("cannot forget the applied migrations")?;
    MIGRATIONS
        .to_latest(&mut conn)
        .await
        .map_err(|(applied, err)| {
            anyhow::anyhow!(
                "cannot create the read models again (after {applied} migrations): {err}"
            )
        })?;
    Ok(())
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
