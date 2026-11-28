//! What admin knows of the pictures of the recipes: each one that is used,
//! and how many times.
//!
//! A picture is stored once however many recipes show it: two users who
//! import the same recipe share its picture. Which recipes those are is
//! read from the recipes ([`crate::recipes::showing`]).
//!
//! A picture is a projection of its stream ([`ImageState`]) whose snapshot
//! is its row of `images` and one row of `image_uses` per use it has left,
//! kept by a projection subscription ([`subscription`]): a picture whose
//! last use was released loses its rows. One picture is loaded from its
//! rows and the events written after them ([`get`]).

use std::collections::BTreeSet;

use evento::Executor;
use evento::metadata::Event;
use evento::projection::{Context, Projection, ProjectionSubscription, Snapshot};
use imkitchen_aggregates::image::{Image, Released, Stored, Used};
use timada_core::db::{FromRow, execute};

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-admin/images";

/// A picture as its stream leaves it.
#[evento::projection]
#[derive(Debug)]
pub struct ImageState {
    /// What its files are named after.
    pub file_id: String,
    /// Of the picture as it was given, in pixels.
    pub width: u32,
    pub height: u32,
    /// What its files weigh, all sizes together.
    pub bytes: u64,
    /// Unix seconds of the last time it was stored.
    pub stored_at: u64,
    /// The uses it has left: more than one, it is shared.
    pub uses: BTreeSet<String>,
}

fn projection<E: Executor>(db: Db) -> Projection<E, ImageState> {
    Projection::new::<Image>()
        .handler(on_stored())
        .handler(on_used())
        .handler(on_released())
        .data(db)
}

/// Keeps the rows of each picture in `db` as its stream leaves it.
pub fn subscription<E: Executor>(db: Db) -> ProjectionSubscription<E, ImageState> {
    projection(db).subscription(SUBSCRIPTION)
}

/// Stored again after its last use was released: other files.
#[evento::handler]
async fn on_stored(event: Event<Stored>, image: &mut ImageState) -> anyhow::Result<()> {
    let picture = &event.data.picture;
    image.file_id = event.data.file_id.to_owned();
    image.width = picture.width;
    image.height = picture.height;
    image.bytes = picture.listing.size + picture.page.size;
    image.stored_at = event.timestamp;
    image.uses.insert(event.data.use_id.to_owned());
    Ok(())
}

#[evento::handler]
async fn on_used(event: Event<Used>, image: &mut ImageState) -> anyhow::Result<()> {
    image.uses.insert(event.data.use_id.to_owned());
    Ok(())
}

#[evento::handler]
async fn on_released(event: Event<Released>, image: &mut ImageState) -> anyhow::Result<()> {
    image.uses.remove(&event.data.use_id);
    Ok(())
}

/// The row of a picture, and how far its stream was folded into it.
#[derive(FromRow)]
struct Row {
    file_id: String,
    width: i64,
    height: i64,
    bytes: i64,
    stored_at: i64,
    cursor: String,
    aggregate_version: i64,
}

#[derive(FromRow)]
struct Use {
    use_id: String,
}

impl<E: Executor> Snapshot<E> for ImageState {
    async fn restore(context: &Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let db: Db = context.extract();
        let image_id = context.id.as_str();
        // The row and its uses as they stood at one moment.
        db.read(image_id, |conn, image_id| {
            Box::pin(async move {
                let row: Option<Row> = timada_core::db::fetch_optional(
                    conn,
                    "SELECT file_id, width, height, bytes, stored_at, cursor, aggregate_version
                     FROM images WHERE image_id = ?1",
                    (image_id,),
                )
                .await?;
                let Some(row) = row else {
                    return Ok(None);
                };
                let uses: Vec<Use> = timada_core::db::fetch_all(
                    conn,
                    "SELECT use_id FROM image_uses WHERE image_id = ?1",
                    (image_id,),
                )
                .await?;
                Ok(Some(ImageState {
                    file_id: row.file_id,
                    width: row.width as u32,
                    height: row.height as u32,
                    bytes: row.bytes as u64,
                    stored_at: row.stored_at as u64,
                    uses: uses.into_iter().map(|one| one.use_id).collect(),
                    cursor: row.cursor,
                    aggregate_version: row.aggregate_version as u16,
                    ..Default::default()
                }))
            })
        })
        .await
    }

    /// A picture with no use left loses its rows: loading it again folds
    /// its stream from the start.
    async fn take_snapshot(&self, context: &Context<'_, E>) -> anyhow::Result<()> {
        let db: Db = context.extract();
        db.write(&(context.id.as_str(), self), |tx, (image_id, image)| {
            Box::pin(async move {
                execute(
                    tx,
                    "DELETE FROM image_uses WHERE image_id = ?1",
                    (*image_id,),
                )
                .await?;
                if image.uses.is_empty() {
                    execute(tx, "DELETE FROM images WHERE image_id = ?1", (*image_id,)).await?;
                    return Ok(());
                }
                execute(
                    tx,
                    "INSERT INTO images (image_id, file_id, width, height, bytes, stored_at,
                        cursor, aggregate_version)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT (image_id) DO UPDATE SET
                        file_id = excluded.file_id, width = excluded.width,
                        height = excluded.height, bytes = excluded.bytes,
                        stored_at = excluded.stored_at, cursor = excluded.cursor,
                        aggregate_version = excluded.aggregate_version",
                    (
                        *image_id,
                        image.file_id.as_str(),
                        image.width,
                        image.height,
                        image.bytes as i64,
                        image.stored_at as i64,
                        image.cursor.as_str(),
                        i64::from(image.aggregate_version),
                    ),
                )
                .await?;
                for use_id in &image.uses {
                    execute(
                        tx,
                        "INSERT INTO image_uses (image_id, use_id) VALUES (?1, ?2)",
                        (*image_id, use_id.as_str()),
                    )
                    .await?;
                }
                Ok(())
            })
        })
        .await
    }
}

/// A picture that is used.
#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ImageRow {
    /// The SHA-256 of the bytes it was given as, in lowercase hex.
    pub image_id: String,
    /// What its files are named after.
    pub file_id: String,
    /// Of the picture as it was given, in pixels.
    pub width: i64,
    pub height: i64,
    /// What its files weigh, all sizes together.
    pub bytes: i64,
    /// Unix seconds.
    pub stored_at: i64,
    /// How many uses it has: more than one, it is shared.
    pub uses: i64,
}

const ROW: &str = "SELECT i.image_id, i.file_id, i.width, i.height, i.bytes, i.stored_at,
        (SELECT count(*) FROM image_uses u WHERE u.image_id = i.image_id) AS uses
    FROM images i";

/// How many pictures are used, as far as admin has followed: only those
/// used more than once when `shared`.
pub async fn count(db: &Db, shared: bool) -> anyhow::Result<u64> {
    let count: i64 = db
        .fetch_scalar(
            "SELECT count(*) FROM images i WHERE ?1 = 0
                OR (SELECT count(*) FROM image_uses u WHERE u.image_id = i.image_id) > 1",
            (shared,),
        )
        .await?;
    Ok(count.max(0) as u64)
}

/// The pictures that are used, the most used first, then the most recent:
/// `limit` of them from the `offset`th, only those used more than once when
/// `shared`.
pub async fn list(db: &Db, shared: bool, offset: i64, limit: i64) -> anyhow::Result<Vec<ImageRow>> {
    db.fetch_all(
        &format!(
            "SELECT * FROM ({ROW}) WHERE ?1 = 0 OR uses > 1
             ORDER BY uses DESC, stored_at DESC, image_id DESC LIMIT ?2 OFFSET ?3"
        ),
        (shared, limit, offset),
    )
    .await
}

/// The picture `image_id`, if it is used, as its rows hold it: a moment
/// behind the commands.
pub async fn row(db: &Db, image_id: &str) -> anyhow::Result<Option<ImageRow>> {
    db.fetch_optional(&format!("{ROW} WHERE i.image_id = ?1"), (image_id,))
        .await
}

/// The picture `image_id`, if it is used, as its stream leaves it: its rows
/// and the events written after them, read from the api.
pub async fn get<E: Executor>(
    executor: &E,
    db: &Db,
    image_id: &str,
) -> anyhow::Result<Option<ImageRow>> {
    let image = projection(db.clone())
        .load(image_id)
        .execute(executor)
        .await?;
    Ok(image
        .filter(|image| !image.uses.is_empty())
        .map(|image| ImageRow {
            image_id: image_id.to_owned(),
            file_id: image.file_id,
            width: i64::from(image.width),
            height: i64::from(image.height),
            bytes: image.bytes as i64,
            stored_at: image.stored_at as i64,
            uses: image.uses.len() as i64,
        }))
}
