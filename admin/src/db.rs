//! The store of admin: one Turso file of its own.
//!
//! admin owns no aggregate and writes no event. Its file holds what it derives
//! from the api's events: the read models its pages are read from ([`Db`])
//! and, in evento's tables next to them, the cursor of each subscription
//! that fills them. Nothing in it is written anywhere else, so a file that
//! is lost is built again from the api, and each replica of admin keeps its
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

/// The tables of admin, next to evento's in the same file. Append only: a
/// migration that was released is never edited.
pub const MIGRATIONS: Migrations<'static> = Migrations::new("imkitchen-admin", &APPLIED);

/// Each migration of admin, in the order they are applied.
const APPLIED: [Migration<'static>; 4] = [
        Migration::up(
            "0001_users",
            // One row per user whose account is not deleted, and nothing of
            // who they are: how many there are is all admin showed of them.
            "CREATE TABLE users (id TEXT NOT NULL PRIMARY KEY);",
        ),
        Migration::up(
            "0002_read_models",
            // `users`: one row per user, deleted or not, with how they sign
            // in; `search` is their address and their name, lowercased and
            // without accents. `user_sessions`: one row per session.
            //
            // `recipes`: one row per recipe that is not deleted, with all
            // it says (`ingredients` and `instructions` as JSON) and the
            // picture it shows; `search` is its title and its ingredients,
            // lowercased and without accents.
            //
            // `recipe_imports`: one row per try at importing a recipe, with
            // the recipe it gave or the reason it gave none; `host` is the
            // site of `url`, what the sources are counted by.
            //
            // `images`: one row per picture that is used, and `image_uses`
            // one per use of it.
            //
            // `contact_messages`: one row per message written to imkitchen.
            "DROP TABLE users;
            CREATE TABLE users (
                id TEXT NOT NULL PRIMARY KEY,
                account TEXT NOT NULL,
                label TEXT NOT NULL,
                is_admin INTEGER NOT NULL,
                deleted INTEGER NOT NULL,
                registered_at INTEGER NOT NULL,
                last_sign_in_at INTEGER,
                sign_in_count INTEGER NOT NULL,
                oauth2_count INTEGER NOT NULL,
                passkey_count INTEGER NOT NULL,
                has_password INTEGER NOT NULL,
                search TEXT NOT NULL
            );
            CREATE INDEX users_listed ON users (deleted, label COLLATE NOCASE, id);
            CREATE TABLE user_sessions (
                id TEXT NOT NULL PRIMARY KEY,
                user_id TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                started_at_ms INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                ended INTEGER NOT NULL,
                user_agent TEXT
            );
            CREATE INDEX user_sessions_of ON user_sessions (user_id, started_at DESC, id);
            CREATE TABLE recipes (
                id TEXT NOT NULL PRIMARY KEY,
                owner_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                ingredients TEXT NOT NULL,
                instructions TEXT NOT NULL,
                prep_minutes INTEGER NOT NULL,
                cook_minutes INTEGER NOT NULL,
                servings INTEGER NOT NULL,
                image_id TEXT,
                file_id TEXT,
                search TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX recipes_listed ON recipes (created_at DESC, id DESC);
            CREATE INDEX recipes_owned ON recipes (owner_id, created_at DESC, id DESC);
            CREATE INDEX recipes_shown ON recipes (image_id);
            CREATE TABLE recipe_imports (
                id TEXT NOT NULL PRIMARY KEY,
                user_id TEXT NOT NULL,
                url TEXT NOT NULL,
                host TEXT NOT NULL,
                page_sent INTEGER NOT NULL,
                recipe_id TEXT,
                reason TEXT,
                at INTEGER NOT NULL
            );
            CREATE INDEX recipe_imports_listed ON recipe_imports (at DESC, id DESC);
            CREATE INDEX recipe_imports_from ON recipe_imports (host, at DESC, id DESC);
            CREATE INDEX recipe_imports_of ON recipe_imports (recipe_id);
            CREATE TABLE images (
                image_id TEXT NOT NULL PRIMARY KEY,
                file_id TEXT NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                bytes INTEGER NOT NULL,
                stored_at INTEGER NOT NULL
            );
            CREATE INDEX images_listed ON images (stored_at DESC, image_id DESC);
            CREATE TABLE image_uses (
                image_id TEXT NOT NULL,
                use_id TEXT NOT NULL,
                PRIMARY KEY (image_id, use_id)
            );
            CREATE TABLE contact_messages (
                id TEXT NOT NULL PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                message TEXT NOT NULL,
                language TEXT NOT NULL,
                sent_at INTEGER NOT NULL
            );
            CREATE INDEX contact_messages_listed ON contact_messages (sent_at DESC, id DESC);",
        ),
        Migration::up(
            "0003_suspensions",
            // One row per user who is suspended: since when, and by which
            // administrator.
            "CREATE TABLE user_suspensions (
                user_id TEXT NOT NULL PRIMARY KEY,
                by_id TEXT NOT NULL,
                at INTEGER NOT NULL
            );",
        ),
        Migration::up(
            "0004_rows_are_snapshots",
            // The rows of `recipes`, and those of `images` with their
            // `image_uses`, are the snapshots of projections: what they
            // hold, and how far their stream was folded into them (`cursor`,
            // and the version of the stream there, `aggregate_version`).
            //
            // The rows that were there say nothing of where their streams
            // stood: the tables are made again and their subscriptions start
            // from the first event, their cursors forgotten.
            "DROP TABLE recipes;
            CREATE TABLE recipes (
                id TEXT NOT NULL PRIMARY KEY,
                owner_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                ingredients TEXT NOT NULL,
                instructions TEXT NOT NULL,
                prep_minutes INTEGER NOT NULL,
                cook_minutes INTEGER NOT NULL,
                servings INTEGER NOT NULL,
                image_id TEXT,
                file_id TEXT,
                search TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                cursor TEXT NOT NULL,
                aggregate_version INTEGER NOT NULL
            );
            CREATE INDEX recipes_listed ON recipes (created_at DESC, id DESC);
            CREATE INDEX recipes_owned ON recipes (owner_id, created_at DESC, id DESC);
            CREATE INDEX recipes_shown ON recipes (image_id);
            DROP TABLE images;
            CREATE TABLE images (
                image_id TEXT NOT NULL PRIMARY KEY,
                file_id TEXT NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                bytes INTEGER NOT NULL,
                stored_at INTEGER NOT NULL,
                cursor TEXT NOT NULL,
                aggregate_version INTEGER NOT NULL
            );
            CREATE INDEX images_listed ON images (stored_at DESC, image_id DESC);
            DELETE FROM image_uses;
            DELETE FROM subscriber WHERE key IN ('imkitchen-admin/recipes', 'imkitchen-admin/images');
            DELETE FROM dead_letter
                WHERE subscriber IN ('imkitchen-admin/recipes', 'imkitchen-admin/images');",
        ),
];

/// Every list of migrations the store's schema is made of.
const SCHEMA: [&Migrations<'static>; 2] = [&evento::turso::MIGRATIONS, &MIGRATIONS];

/// Creates the store at `path` if needed, file and directory, and applies
/// the pending migrations: evento's, then admin's own.
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
        "there is no store at {}: run `imkitchen-admin migrate` first",
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
        "the store at {} has migrations to apply ({}): run `imkitchen-admin migrate` first",
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
        let path = dir.path().join("nested").join("admin.db");

        let Err(err) = open(&path).await else {
            panic!("opened a missing store");
        };
        assert!(err.to_string().contains("migrate"), "{err}");

        drop(migrate(&path).await.unwrap());
        // Again: nothing is pending.
        drop(migrate(&path).await.unwrap());
        let db = Db::from(open(&path).await.unwrap());
        assert_eq!(crate::users::count(&db).await.unwrap(), 0);
        assert_eq!(crate::recipes::count(&db, None).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn a_store_whose_rows_were_no_snapshots_follows_them_again_from_the_start() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("admin.db");
        // The store as admin left it before its rows were snapshots: rows in
        // the tables, and the subscriptions past the events that wrote them.
        let store = Turso::open_concurrent(&path).await.unwrap();
        let mut conn = store.connect().unwrap();
        let before = Migrations::new("imkitchen-admin", &APPLIED[..3]);
        before.to_latest(&mut conn).await.unwrap();
        let db = Db::from(store);
        db.execute(
            "INSERT INTO image_uses (image_id, use_id) VALUES ('picture', 'use')",
            (),
        )
        .await
        .unwrap();
        for key in [
            crate::recipes::SUBSCRIPTION,
            crate::images::SUBSCRIPTION,
            crate::users::SUBSCRIPTION,
        ] {
            db.execute(
                "INSERT INTO subscriber (key, worker_id, cursor, lag, created_at)
                 VALUES (?1, 'worker', 'cursor', 0, 0)",
                (key,),
            )
            .await
            .unwrap();
        }
        drop((conn, db));

        let db = Db::from(migrate(&path).await.unwrap());
        let uses: i64 = db
            .fetch_scalar("SELECT count(*) FROM image_uses", ())
            .await
            .unwrap();
        assert_eq!(uses, 0);
        // The recipes and the pictures start again; the users go on.
        let cursors: Vec<(String,)> = db
            .fetch_all("SELECT key FROM subscriber", ())
            .await
            .unwrap();
        assert_eq!(cursors, [(crate::users::SUBSCRIPTION.to_owned(),)]);
    }
}
