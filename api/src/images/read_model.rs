//! What this node knows of the pictures' files: the `images` table, and the
//! subscription that keeps it, and the files with it, in step with the
//! pictures' streams.
//!
//! One row per picture that is used: the id its files are named after and
//! what they are. A handler reloads the picture from its stream and sets
//! the node to what it says, so an event delivered twice, or long after the
//! fact on a node that was away, leaves the node as it should be: the files
//! of a picture nobody uses any more are deleted with its row, and those of
//! a picture this node lacks are asked of a peer. Files are on a node
//! before the event that names them, and may be let go of before this node
//! gets to that event: so the files an event names go when the picture no
//! longer has them, whether this node knew of them or not.
//!
//! Nothing here fails for a peer: a file that could not be had is asked for
//! again when somebody wants to see it ([`super::image`]).
//!
//! Every node runs the subscription over its own file, with a cursor of its
//! own.

use std::sync::Arc;

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::image::{Released, Stored, Used};
use timada_core::db::{FromRow, Json};
use timada_core::{Picture, Rendition};

use super::model::{self, Files};
use super::store::{Fetch, Images};
use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-api/images";

/// Follows the pictures into `db` and onto this node's disk. Start it over
/// a handle that reads this node's file as it stands: what a handler loads
/// is behind the event it handles.
pub fn subscription<E: evento::Executor>(db: Db, images: Arc<Images>) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .data(images)
        .handler(on_stored())
        .handler(on_used())
        .handler(on_released())
}

#[evento::subscription]
async fn on_stored<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Stored>,
) -> anyhow::Result<()> {
    settle(context, &event.aggregate_id, Some(&event.data.file_id)).await
}

#[evento::subscription]
async fn on_used<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Used>,
) -> anyhow::Result<()> {
    settle(context, &event.aggregate_id, None).await
}

#[evento::subscription]
async fn on_released<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Released>,
) -> anyhow::Result<()> {
    settle(context, &event.aggregate_id, None).await
}

/// Sets this node to what the stream of the picture `image_id` says: its
/// files known and held while it has some, and no others. `named` is the
/// id of the files the event at hand stored, which this node may hold
/// without knowing of them.
async fn settle<E: evento::Executor>(
    context: &Context<'_, E>,
    image_id: &str,
    named: Option<&str>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let images: Arc<Images> = context.extract();
    let files = model::load(context.executor, image_id)
        .await?
        .and_then(|image| image.files);
    let current = |file_id: &str| files.as_ref().is_some_and(|files| files.id == file_id);

    // The files this node knows the picture by, and those the event names:
    // all but the ones it has now go.
    let known: Vec<Known> = db
        .fetch_all(
            "SELECT file_id FROM images WHERE image_id = ?1",
            (image_id,),
        )
        .await?;
    let named = named.map(str::to_owned);
    for file_id in known.into_iter().map(|known| known.file_id).chain(named) {
        if current(&file_id) {
            continue;
        }
        db.execute("DELETE FROM images WHERE file_id = ?1", (file_id.as_str(),))
            .await?;
        images.remove(&file_id).await;
    }
    let Some(files) = files else {
        return Ok(());
    };
    // Known before it is fetched: a miss meanwhile asks a peer for it too.
    db.execute(
        "INSERT INTO images (file_id, image_id, picture) VALUES (?1, ?2, ?3)
         ON CONFLICT (file_id) DO NOTHING",
        (files.id.as_str(), image_id, Json(files.picture).to_text()?),
    )
    .await?;
    // Nothing is asked of a peer for the files this node holds.
    fetch(&images, &files).await;
    Ok(())
}

#[derive(FromRow)]
struct Known {
    file_id: String,
}

/// Takes from the peers the files of a picture this node lacks. One that
/// could not be had is said and left for whoever asks to see it.
async fn fetch(images: &Images, files: &Files) {
    for (rendition, recorded) in files.picture.files() {
        let file = Picture::file(&files.id, rendition);
        match images.fetch_rendition(&files.id, rendition, recorded).await {
            Ok(Fetch::Held | Fetch::Fetched) => {}
            Ok(Fetch::Gone) => tracing::warn!(%file, "no node holds a picture the store records"),
            Ok(Fetch::Unreachable(reason)) => {
                tracing::info!(%file, reason, "a picture was not fetched yet");
            }
            Err(err) => {
                tracing::warn!(%file, err = format!("{err:#}"), "a picture could not be fetched");
            }
        }
    }
}

#[derive(FromRow)]
struct Recorded {
    picture: Json<Picture>,
}

/// One rendition as this node holds it, or as a peer does when this node
/// lacks a file the store records: fetched, it is kept. `None` for a file
/// the store does not know, which costs the peers nothing, and for one that
/// could not be had.
pub async fn rendition(
    db: &Db,
    images: &Images,
    rendition: Rendition,
    file_id: &str,
) -> anyhow::Result<Option<bytes::Bytes>> {
    if let Some(bytes) = images.read(rendition, file_id).await? {
        return Ok(Some(bytes));
    }
    let recorded: Option<Recorded> = db
        .fetch_optional("SELECT picture FROM images WHERE file_id = ?1", (file_id,))
        .await?;
    let Some(Recorded { picture }) = recorded else {
        return Ok(None);
    };
    match images
        .fetch_rendition(file_id, rendition, picture.0.rendition(rendition))
        .await?
    {
        Fetch::Held | Fetch::Fetched => images.read(rendition, file_id).await,
        Fetch::Gone => Ok(None),
        Fetch::Unreachable(reason) => {
            tracing::info!(
                file_id,
                reason,
                "a picture this node lacks could not be fetched"
            );
            Ok(None)
        }
    }
}
