use std::io::Cursor;
use std::str::FromStr;

use evento::Sqlite;
use evento::migrator::{Migrate, Plan};
use image::{DynamicImage, ImageFormat, RgbImage};
use imkitchen_core::State;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use temp_dir::TempDir;

async fn setup_test_state(path: std::path::PathBuf) -> anyhow::Result<State<Sqlite>> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    let mut conn = pool.acquire().await?;
    imkitchen_db::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;

    Ok(State {
        executor: pool.clone().into(),
        read_db: pool.clone(),
        write_db: pool,
    })
}

fn png_bytes() -> Vec<u8> {
    let img = RgbImage::new(4, 4);
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// Uploading a thumbnail must stash the original bytes transiently in
/// recipe_thumbnail (device='original') for the async resizer, and the
/// committed ThumbnailUploaded event must carry no image bytes.
#[tokio::test]
async fn upload_thumbnail_stashes_original_and_emits_byte_free_event() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = setup_test_state(dir.child("db.sqlite3")).await?;
    let recipe = imkitchen_core::recipe::Module::new(state.clone());

    let id = recipe.create("chef-1", None).await?;

    let png = png_bytes();
    recipe.upload_thumbnail(&id, png.clone(), "chef-1").await?;

    // The original is stashed under device='original' with the exact bytes.
    let original: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT data FROM recipe_thumbnail WHERE id = ? AND device = 'original'",
    )
    .bind(&id)
    .fetch_optional(&state.read_db)
    .await?;
    assert_eq!(original.as_deref(), Some(png.as_slice()));

    // The ThumbnailUploaded event exists but carries no image bytes.
    let uploaded_len: Option<i64> = sqlx::query_scalar(
        "SELECT length(data) FROM event WHERE aggregator_id = ? AND name = 'ThumbnailUploaded'",
    )
    .bind(&id)
    .fetch_optional(&state.read_db)
    .await?;
    assert_eq!(uploaded_len, Some(0), "event must be byte-free");

    Ok(())
}

/// Full async pipeline: upload -> resize subscription writes the three variants
/// to recipe_thumbnail (authoritative), emits byte-free ThumbnailResized
/// markers, deletes the transient original; the user projection derives the
/// blur placeholder from the stored mobile variant.
#[tokio::test]
async fn resize_pipeline_writes_variants_and_blur_without_event_bytes() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    {
        let mut conn = pool.acquire().await?;
        imkitchen_db::migrator::<sqlx::Sqlite>()?
            .run(&mut conn, &Plan::apply_all())
            .await?;
    }

    let rw: evento::sql::RwSqlite = (
        evento::Sqlite::from(pool.clone()),
        evento::Sqlite::from(pool.clone()),
    )
        .into();
    let executor = evento::Evento::new(rw);

    // Mirror server.rs wiring for the three recipe subscriptions.
    let _sub_command = imkitchen_core::recipe::subscription()
        .data((pool.clone(), pool.clone()))
        .start(&executor)
        .await?;
    let _sub_query = imkitchen_core::recipe::query::user::create_projection()
        .data((pool.clone(), pool.clone()))
        .subscription("recipe-query")
        .all()
        .start(&executor)
        .await?;
    let _sub_thumbnail = imkitchen_core::recipe::query::thumbnail::subscription()
        .data(pool.clone())
        .all()
        .start(&executor)
        .await?;

    let state = State {
        executor: executor.clone(),
        read_db: pool.clone(),
        write_db: pool.clone(),
    };
    let recipe = imkitchen_core::recipe::Module::new(state);

    let id = recipe.create("chef-1", None).await?;
    recipe.upload_thumbnail(&id, png_bytes(), "chef-1").await?;

    // Poll until the async resize subscription has produced the three variants.
    let mut variants: Vec<String> = Vec::new();
    for _ in 0..100 {
        variants = sqlx::query_scalar(
            "SELECT device FROM recipe_thumbnail WHERE id = ? AND device <> 'original' ORDER BY device",
        )
        .bind(&id)
        .fetch_all(&pool)
        .await?;
        if variants.len() == 3 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert_eq!(
        variants,
        vec![
            "desktop".to_string(),
            "mobile".to_string(),
            "tablet".to_string()
        ],
        "all three device variants written to recipe_thumbnail"
    );

    // The transient original is cleaned up.
    let original: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT data FROM recipe_thumbnail WHERE id = ? AND device = 'original'",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?;
    assert!(original.is_none(), "transient original must be deleted");

    // ThumbnailResized markers carry only the device string (no image bytes):
    // 7 bytes for mobile/tablet, 8 for desktop.
    let bad_len: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM event WHERE aggregator_id = ? AND name = 'ThumbnailResized' AND length(data) NOT IN (7, 8)",
    )
    .bind(&id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(
        bad_len, 0,
        "ThumbnailResized events must be byte-free markers"
    );

    // The user projection derived a blur placeholder from the mobile variant.
    let mut blur: Option<String> = None;
    for _ in 0..100 {
        blur = sqlx::query_scalar("SELECT blur_placeholder FROM recipe_user WHERE id = ?")
            .bind(&id)
            .fetch_optional(&pool)
            .await?
            .flatten();
        if blur.is_some() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let blur = blur.expect("blur placeholder derived from the stored mobile variant");
    assert!(
        blur.starts_with("data:image/webp;base64,"),
        "blur placeholder is a webp data URL, got: {blur}"
    );

    Ok(())
}
