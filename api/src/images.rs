//! The pictures of the recipes: stored once each, kept on every node, and
//! served from any of them.
//!
//! A picture is known by the bytes it was given as: whoever gives the same
//! bytes, by uploading them or by importing the same recipe, uses the one
//! picture already stored ([`attach`]). It is kept as long as one recipe
//! shows it ([`release`]). What the store records of it is in
//! `imkitchen-aggregates` (`image::Image`); its files are on the nodes'
//! disks ([`Images`]), copied between them on a port of their own
//! ([`port`]).
//!
//! `GET /v1/images/{size}/{file}` serves one, in the size asked for. It
//! takes no token: a picture is fetched by whatever shows it (an `<img>`,
//! a phone's image view), which sends none, and its address is only handed
//! out with the recipe that shows it, under a name nobody guesses.

mod disk;
mod model;
mod peers;
pub mod port;
mod read_model;
mod release;
mod store;

use anyhow::Context as _;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use evento::cluster::{NodeTls, Settings};
use timada_core::RENDITION_MIME;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub use self::model::{attach, release};
pub use self::peers::Peers;
pub use self::read_model::{SUBSCRIPTION, subscription};
pub use self::release::{
    SUBSCRIPTION as RELEASE_SUBSCRIPTION, subscription as release_subscription,
};
pub use self::store::{
    IMAGE_TOO_LARGE, IMAGE_TOO_MANY_PIXELS, Images, MAX_IMAGE_BYTES, UNSUPPORTED_IMAGE,
};

/// No picture is stored under this id: none ever was, or no recipe shows
/// it any more.
pub const IMAGE_NOT_FOUND: &str = "image_not_found";
use crate::config::ImagesConfig;
use crate::error::{ApiError, NOT_FOUND};
use crate::openapi::IMAGES;
use crate::server::AppState;

/// How long a browser or a cache may keep a picture. The file at an address
/// never changes (another picture is another address), so: as long as it
/// likes.
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// What a cache may keep of a picture this node could not serve: nothing.
/// The file may be on its way from another node, and a miss kept for a year
/// would hide it once it lands.
const NO_STORE: &str = "no-store";

/// The pictures of this node, as its configuration says: in `images.dir`,
/// and on the other nodes of `cluster` when it names some, reached with
/// `tls` when the nodes talk over it.
pub fn open(
    config: &ImagesConfig,
    cluster: Option<&Settings>,
    tls: Option<NodeTls>,
) -> anyhow::Result<Images> {
    let peers = match cluster.filter(|cluster| cluster.peers.len() > 1) {
        Some(cluster) => {
            let others = config
                .peers
                .iter()
                .enumerate()
                .filter(|(index, _)| *index as u64 != cluster.node_id)
                .map(|(_, url)| url.clone())
                .collect();
            let peers = Peers::new(others, cluster.peers.len(), tls)
                .context("cannot reach the other nodes for the pictures (images.peers)")?;
            Some(peers)
        }
        None => None,
    };
    Ok(Images::new(&config.dir, peers))
}

/// The address a picture is served at, from the root of the api: `size` is
/// the directory of one of its two sizes.
pub fn path(size: &str, file_id: &str) -> String {
    format!("{}/images/{size}/{file_id}.webp", crate::openapi::VERSION)
}

/// Whether the picture `image_id` is stored: one recipe at least shows it.
pub async fn is_stored<E: evento::Executor>(executor: &E, image_id: &str) -> anyhow::Result<bool> {
    let image = model::load(executor, image_id).await?;
    Ok(image.is_some_and(|image| image.files.is_some()))
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(image))
}

/// A picture of a recipe
///
/// The addresses of a recipe's picture come with the recipe (`image`), one
/// per size it is kept in. A picture never changes at its address, and may
/// be cached for good. No token is needed.
#[utoipa::path(
    get,
    path = "/images/{size}/{file}",
    tag = IMAGES,
    params(
        ("size" = String, Path, description = "`400` or `1200`: the most pixels of its longest side"),
        ("file" = String, Path, description = "The name of the picture, as a recipe gives it"),
    ),
    responses((status = OK, description = "The picture", content_type = "image/webp", body = Vec<u8>))
)]
async fn image(
    State(state): State<AppState>,
    Path((size, file)): Path<(String, String)>,
) -> Response {
    let found = match store::parse_file(&size, &file) {
        Some((rendition, file_id)) => {
            read_model::rendition(&state.db, &state.images, rendition, &file_id).await
        }
        None => Ok(None),
    };
    match found {
        Ok(Some(bytes)) => (
            [(CONTENT_TYPE, RENDITION_MIME), (CACHE_CONTROL, IMMUTABLE)],
            bytes,
        )
            .into_response(),
        Ok(None) => missed(ApiError::refused(StatusCode::NOT_FOUND, NOT_FOUND)),
        Err(err) => missed(err.into()),
    }
}

/// A refusal no cache keeps.
fn missed(refusal: ApiError) -> Response {
    ([(CACHE_CONTROL, NO_STORE)], refusal).into_response()
}
