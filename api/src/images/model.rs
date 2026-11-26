//! The write side of a picture: what its stream says of it, and the two
//! things done to it, using it once more and releasing a use.
//!
//! A picture's stream is under the digest of the bytes it was given as, so
//! whoever gives the same bytes finds it: two users who import the same
//! recipe use the one picture, stored once. Each use is written here before
//! a recipe names it ([`attach`]) and released once the recipe let go of it
//! ([`release`], by [`super::release`]); the files are kept as long as one
//! use is left.
//!
//! Using and releasing go out at the version the stream was read at, so
//! the two never cross: a use lands before the release that would have been
//! the last, and the files stay, or after it, and the picture is stored
//! again under a fresh name. A file's name is never given twice, which is
//! what lets every node delete the files of a picture nobody uses without
//! asking whether somebody is about to.
//!
//! A node may die between a use written here and the recipe that names it.
//! Such a use is never released: the picture it holds stays stored, which
//! costs its two files and nothing else.

use std::collections::HashSet;

use axum::http::StatusCode;
use bytes::Bytes;
use evento::metadata::Event;
use evento::{Executor, Projection, ProjectionAggregate, WriteError};
use imkitchen_aggregates::image::{Image, RecipeImage, Released, Stored, Used};
use timada_core::Picture;
use ulid::Ulid;

use super::store::{self, Images, Rendered};
use crate::error::{ApiError, ApiResult};

/// How many times a write that lost to another on the same picture is made
/// again, over the picture as the other left it.
const WRITE_ATTEMPTS: u32 = 5;

/// How many events a stream takes before the uses of its picture go to the
/// next one. A stream holds 65 535 versions, and each use written by then
/// still has its release to write: half of them.
const STREAM_FULL: u16 = 30_000;

/// How many streams one picture may fill: past them, it is not taken.
const STREAMS_MAX: u32 = 16;

/// The files that hold a picture, and what they are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Files {
    /// What the files are named after.
    pub id: String,
    pub picture: Picture,
}

/// A picture as its stream leaves it.
#[evento::projection(id = id, name = "imkitchen/ImageState")]
#[evento::snapshot(none)]
#[derive(Debug, PartialEq)]
pub struct ImageState {
    pub id: String,
    /// The files that hold it; `None` once nobody uses it and until it is
    /// stored again.
    pub files: Option<Files>,
    pub uses: HashSet<String>,
}

fn projection<E: Executor>() -> Projection<E, ImageState> {
    Projection::new::<Image>()
        .handler(on_stored())
        .handler(on_used())
        .handler(on_released())
        .strict()
}

#[evento::handler]
async fn on_stored(event: Event<Stored>, image: &mut ImageState) -> anyhow::Result<()> {
    image.id = event.aggregate_id.to_owned();
    image.files = Some(Files {
        id: event.data.file_id.to_owned(),
        picture: event.data.picture,
    });
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
    if image.uses.is_empty() {
        image.files = None;
    }
    Ok(())
}

/// The picture `id` as its stream leaves it.
pub async fn load<E: Executor>(executor: &E, id: &str) -> anyhow::Result<Option<ImageState>> {
    projection().load(id).execute(executor).await
}

/// The stream of the picture whose bytes have `digest`: the digest itself,
/// then one stream after another as they fill.
fn stream_id(digest: &str, nth: u32) -> String {
    match nth {
        0 => digest.to_owned(),
        nth => evento::hash_ids(vec![digest.to_owned(), nth.to_string()]),
    }
}

/// Uses the picture `bytes` are once more, and answers the use for a recipe
/// to name. Bytes seen before are the picture already stored: nothing is
/// rendered and nothing is written to a disk. Otherwise they are rendered
/// and kept on a majority of the nodes first, and refused when they are not
/// a picture.
///
/// Whoever is answered a use either records it in a recipe or gives it back
/// ([`release`]).
pub async fn attach<E: Executor>(
    executor: &E,
    images: &Images,
    bytes: Bytes,
) -> ApiResult<RecipeImage> {
    let digest = store::digest(&bytes);
    let mut rendered: Option<Rendered> = None;
    let mut nth = 0;
    let mut attempts = 0;
    while attempts < WRITE_ATTEMPTS && nth < STREAMS_MAX {
        let image_id = stream_id(&digest, nth);
        let state = load(executor, &image_id).await?;
        let version = match &state {
            Some(state) => state.aggregate_version()?,
            None => 0,
        };
        if version >= STREAM_FULL {
            nth += 1;
            continue;
        }
        attempts += 1;
        let use_id = Ulid::generate().to_string();
        let mut write = evento::append(&image_id);
        write.original_version(version);

        let known = state.and_then(|state| state.files);
        let stored = known.is_none();
        let files = match known {
            Some(files) => {
                write.event(&Used {
                    use_id: use_id.clone(),
                });
                files
            }
            None => {
                let rendered = match &rendered {
                    Some(rendered) => rendered,
                    None => rendered.insert(images.render(bytes.clone()).await??),
                };
                let id = Ulid::generate().to_string();
                let picture = images.keep(&id, rendered).await.map_err(|err| {
                    tracing::warn!(err = format!("{err:#}"), "a picture was not kept");
                    not_taken()
                })?;
                write.event(&Stored {
                    file_id: id.clone(),
                    picture,
                    use_id: use_id.clone(),
                });
                Files { id, picture }
            }
        };
        match write.commit(executor).await {
            Ok(_) => {
                return Ok(RecipeImage {
                    image_id,
                    use_id,
                    file_id: files.id,
                    picture: files.picture,
                });
            }
            // Another use or a release got there first: decided again from
            // what the picture is now. Files kept for nothing go from this
            // node; the copies the others took are theirs to waste.
            Err(WriteError::InvalidOriginalVersion) => {
                if stored {
                    images.remove(&files.id).await;
                }
            }
            Err(err) => return Err(err.into()),
        }
    }
    Err(not_taken())
}

fn not_taken() -> ApiError {
    ApiError::refused(
        StatusCode::SERVICE_UNAVAILABLE,
        timada_core::WRITE_NOT_TAKEN,
    )
}

/// Gives the use `use_id` of the picture `image_id` back; the last one
/// takes the picture's files with it, on every node. A use already given
/// back, or never written, changes nothing.
pub async fn release<E: Executor>(
    executor: &E,
    image_id: &str,
    use_id: &str,
) -> anyhow::Result<()> {
    for _ in 0..WRITE_ATTEMPTS {
        let Some(image) = load(executor, image_id).await? else {
            return Ok(());
        };
        if !image.uses.contains(use_id) {
            return Ok(());
        }
        let released = Released {
            use_id: use_id.to_owned(),
        };
        match image.write()?.event(&released).commit(executor).await {
            Ok(_) => return Ok(()),
            Err(WriteError::InvalidOriginalVersion) => continue,
            Err(err) => return Err(err.into()),
        }
    }
    anyhow::bail!(
        "the use {use_id} of the picture {image_id} was not released: others kept writing it"
    )
}
