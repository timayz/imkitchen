//! What admin knows of the pictures of the recipes: each one that is used,
//! and how many times.
//!
//! A picture is stored once however many recipes show it: two users who
//! import the same recipe share its picture. admin keeps one row per
//! picture that has a use left and one per use; which recipes those are is
//! read from the recipes ([`crate::recipes::showing`]).
//!
//! The handlers leave the tables as they should be when an event is
//! delivered twice (the subscription is at-least-once): a use is there once
//! however often it is written, and gone however often it is released, the
//! picture with its last one.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::image::{Released, Stored, Used};
use timada_core::db::{FromRow, execute};

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-admin/images";

/// Follows the pictures into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_stored())
        .handler(on_used())
        .handler(on_released())
}

const USE: &str = "INSERT INTO image_uses (image_id, use_id) VALUES (?1, ?2)
                   ON CONFLICT (image_id, use_id) DO NOTHING";

#[evento::subscription]
async fn on_stored<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Stored>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    db.write(&event, |tx, event| {
        Box::pin(async move {
            let picture = &event.data.picture;
            // Stored again after its last use was released: other files.
            execute(
                tx,
                "INSERT INTO images (image_id, file_id, width, height, bytes, stored_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT (image_id) DO UPDATE SET
                    file_id = excluded.file_id, width = excluded.width,
                    height = excluded.height, bytes = excluded.bytes,
                    stored_at = excluded.stored_at",
                (
                    event.aggregate_id.as_str(),
                    event.data.file_id.as_str(),
                    picture.width,
                    picture.height,
                    (picture.listing.size + picture.page.size) as i64,
                    event.timestamp as i64,
                ),
            )
            .await?;
            execute(
                tx,
                USE,
                (event.aggregate_id.as_str(), event.data.use_id.as_str()),
            )
            .await?;
            Ok(())
        })
    })
    .await
}

#[evento::subscription]
async fn on_used<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Used>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    db.execute(
        USE,
        (event.aggregate_id.as_str(), event.data.use_id.as_str()),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_released<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Released>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    db.write(&event, |tx, event| {
        Box::pin(async move {
            let image_id = event.aggregate_id.as_str();
            execute(
                tx,
                "DELETE FROM image_uses WHERE image_id = ?1 AND use_id = ?2",
                (image_id, event.data.use_id.as_str()),
            )
            .await?;
            // The last use takes the picture with it.
            execute(
                tx,
                "DELETE FROM images WHERE image_id = ?1
                    AND NOT EXISTS (SELECT 1 FROM image_uses WHERE image_id = ?1)",
                (image_id,),
            )
            .await?;
            Ok(())
        })
    })
    .await
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

/// The picture `image_id`, if it is used.
pub async fn get(db: &Db, image_id: &str) -> anyhow::Result<Option<ImageRow>> {
    db.fetch_optional(&format!("{ROW} WHERE i.image_id = ?1"), (image_id,))
        .await
}
