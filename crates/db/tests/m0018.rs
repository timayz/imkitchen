use evento::migrator::{Migrate, Plan};
use evento::{Aggregate, WriteBuilder};
use imkitchen_types::favorite::{Favorite, Saved, Unsaved};
use sha3::{Digest, Sha3_256};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;
use temp_dir::TempDir;

/// `evento::hash_ids` before 2.0.0-alpha.28: SHA3-256 of the ids joined
/// without a length prefix.
fn old_hash_ids(ids: &[&str]) -> String {
    let mut hasher = Sha3_256::new();
    for id in ids {
        hasher.update(id);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Runs the `m0018` operation on its own: the schema it needs comes from the
/// full migration run, so the legacy events are written afterwards and the
/// operation is replayed over them.
async fn rekey(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;
    sqlx_migrator::Operation::<sqlx::Sqlite>::up(&imkitchen_db::m0018::RekeyFavorites, &mut conn)
        .await?;
    Ok(())
}

async fn stream(pool: &SqlitePool, id: &str) -> anyhow::Result<Vec<(String, i64)>> {
    Ok(sqlx::query_as(
        "SELECT name, version FROM event WHERE aggregator_type = ? AND aggregator_id = ? \
         ORDER BY timestamp, timestamp_subsec, version, id",
    )
    .bind(Favorite::aggregate_type())
    .bind(id)
    .fetch_all(pool)
    .await?)
}

#[tokio::test]
async fn favorites_move_to_the_current_stream_id() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    let mut conn = pool.acquire().await?;
    imkitchen_db::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;
    drop(conn);
    let executor = evento::Sqlite::from(pool.clone());

    let saved = |recipe: &str| Saved {
        recipe_id: recipe.to_owned(),
        recipe_owner: "chef".to_owned(),
    };
    let unsaved = |recipe: &str| Unsaved {
        recipe_id: recipe.to_owned(),
    };

    // Saved and unsaved before alpha.28, never touched since.
    let old_a = old_hash_ids(&["recipe-a", "cook"]);
    WriteBuilder::new(&old_a)
        .event(&saved("recipe-a"))
        .event(&unsaved("recipe-a"))
        .requested_by("cook")
        .commit(&executor)
        .await?;

    // Saved before alpha.28, then saved again on the current id while the
    // old stream was unreachable.
    let old_b = old_hash_ids(&["recipe-b", "cook"]);
    let new_b = evento::hash_ids(vec!["recipe-b", "cook"]);
    WriteBuilder::new(&old_b)
        .event(&saved("recipe-b"))
        .requested_by("cook")
        .commit(&executor)
        .await?;
    WriteBuilder::new(&new_b)
        .event(&saved("recipe-b"))
        .event(&unsaved("recipe-b"))
        .requested_by("cook")
        .commit(&executor)
        .await?;

    // Saved after alpha.28 by another cook: already in place.
    let new_c = evento::hash_ids(vec!["recipe-c", "other"]);
    WriteBuilder::new(&new_c)
        .event(&saved("recipe-c"))
        .requested_by("other")
        .commit(&executor)
        .await?;

    sqlx::query("INSERT INTO snapshot (type, projection, id, cursor, revision, data) VALUES (?, 'p', ?, '', '1', x'')")
        .bind(Favorite::aggregate_type())
        .bind(&new_b)
        .execute(&pool)
        .await?;

    rekey(&pool).await?;

    assert!(stream(&pool, &old_a).await?.is_empty());
    assert_eq!(
        stream(&pool, &evento::hash_ids(vec!["recipe-a", "cook"])).await?,
        vec![("Saved".to_owned(), 1), ("Unsaved".to_owned(), 2)]
    );

    assert!(stream(&pool, &old_b).await?.is_empty());
    assert_eq!(
        stream(&pool, &new_b).await?,
        vec![
            ("Saved".to_owned(), 1),
            ("Saved".to_owned(), 2),
            ("Unsaved".to_owned(), 3)
        ]
    );

    assert_eq!(stream(&pool, &new_c).await?, vec![("Saved".to_owned(), 1)]);

    let snapshots: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snapshot WHERE type = ?")
        .bind(Favorite::aggregate_type())
        .fetch_one(&pool)
        .await?;
    assert_eq!(snapshots, 0);

    // Idempotent: a second pass over the rewritten streams changes nothing.
    let before: Vec<(String, String, i64)> =
        sqlx::query_as("SELECT id, aggregator_id, version FROM event ORDER BY id")
            .fetch_all(&pool)
            .await?;
    rekey(&pool).await?;
    let after: Vec<(String, String, i64)> =
        sqlx::query_as("SELECT id, aggregator_id, version FROM event ORDER BY id")
            .fetch_all(&pool)
            .await?;
    assert_eq!(before, after);

    Ok(())
}
