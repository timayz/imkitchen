use std::collections::BTreeSet;

use evento::Turso;
use imkitchen_api::db;

/// The tables a store holds, by name.
async fn tables(store: &db::Store) -> BTreeSet<String> {
    let conn = store.connect_read().unwrap();
    let mut rows = conn
        .query("SELECT name FROM sqlite_schema WHERE type = 'table'", ())
        .await
        .unwrap();
    let mut tables = BTreeSet::new();
    while let Some(row) = rows.next().await.unwrap() {
        tables.insert(row.get::<String>(0).unwrap());
    }
    tables
}

#[tokio::test]
async fn migrate_creates_a_fresh_store() {
    let dir = tempfile::tempdir().unwrap();
    // Neither the file nor its directory exists yet.
    let path = dir.path().join("data").join("imkitchen.db");

    let store = db::migrate(&path).await.unwrap();

    assert!(path.is_file());
    assert_eq!(
        store.write_mode(),
        evento::turso::WriteMode::Concurrent,
        "the store takes several writers"
    );
    let tables = tables(&store).await;
    for table in [
        "event",
        "snapshot",
        "subscriber",
        "dead_letter",
        "cluster_commands",
        "cluster_meta",
    ] {
        assert!(tables.contains(table), "no `{table}` table in {tables:?}");
    }
}

#[tokio::test]
async fn migrate_twice_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("imkitchen.db");

    let first = tables(&db::migrate(&path).await.unwrap()).await;
    let second = tables(&db::migrate(&path).await.unwrap()).await;

    assert_eq!(first, second);
}

#[tokio::test]
async fn open_keeps_the_store_migrate_left() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("imkitchen.db");
    drop(db::migrate(&path).await.unwrap());

    let store = db::open(&path).await.unwrap();

    assert_eq!(store.write_mode(), evento::turso::WriteMode::Concurrent);
}

#[tokio::test]
async fn open_refuses_a_missing_store() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("imkitchen.db");

    let err = db::open(&path).await.err().unwrap();

    assert!(err.to_string().contains("migrate"), "{err}");
    assert!(!path.exists(), "open created the store");
}

#[tokio::test]
async fn open_refuses_a_store_with_pending_migrations() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("imkitchen.db");
    // The events' tables only: the cluster journal's are still to apply.
    drop(Turso::open_concurrent(&path).await.unwrap());

    let err = db::open(&path).await.err().unwrap();

    assert!(err.to_string().contains("evento-cluster: 1"), "{err}");
}
