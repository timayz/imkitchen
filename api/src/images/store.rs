//! The files of the pictures, on this node's disk and on the other nodes'.
//!
//! What is given as a picture is not kept. It is decoded, bounded to each
//! of the two sizes a picture is shown in ([`Rendition`]) and encoded as
//! WebP ([`Images::render`]), so what every node stores is a few tens of
//! kilobytes per picture whatever the camera made, and what a list sends a
//! phone is the small one. The files sit under `images.dir`, one directory
//! per size, named after the id the picture's stream gives them:
//! `400/<file id>.webp` and `1200/<file id>.webp`.
//!
//! With several nodes every node keeps every picture, so one that goes down
//! takes none with it. The files are copied to the other nodes and only
//! recorded once a majority of nodes hold them ([`Images::keep`]), the same
//! majority a commit needs. A node that was away then takes from a peer
//! what it lacks ([`Images::fetch_rendition`]), checking each copy against
//! the size and digest the store recorded.
//!
//! Every file name is checked before it touches the disk
//! ([`Images::path_of`]): the ids come from events and from addresses, so a
//! name is never trusted to stay inside the directory.

use std::io::Cursor;
use std::path::PathBuf;

use axum::http::StatusCode;
use bytes::Bytes;
use image::{DynamicImage, GenericImageView, ImageDecoder, ImageReader, imageops::FilterType};
use sha2::{Digest, Sha256};
use timada_core::{MAX_PICTURE_PIXELS, Picture, RENDITION_EXT, Rendition, RenditionFile};
use tokio::sync::Semaphore;

use super::disk::{self, Durability};
use super::peers::Peers;
use crate::error::ApiError;

/// Where a node takes and gives a copy of a picture, on the nodes' port.
pub const PEER_ROUTE: &str = "/images";

/// The most a picture may weigh as it is given.
pub const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;

/// The most a rendition may weigh. A 1200 px WebP of a photograph is a few
/// hundred kilobytes at most; a copy heavier than this is not one of ours.
pub const MAX_RENDITION_BYTES: usize = 2 * 1024 * 1024;

/// What is given is not a JPEG, PNG, WebP or GIF picture.
pub const UNSUPPORTED_IMAGE: &str = "unsupported_image";
/// What is given weighs more than [`MAX_IMAGE_BYTES`].
pub const IMAGE_TOO_LARGE: &str = "image_too_large";
/// What is given has more pixels than a picture may.
pub const IMAGE_TOO_MANY_PIXELS: &str = "image_too_many_pixels";

/// How many pictures this node renders at once. Decoding a 25-megapixel
/// photograph takes a hundred megabytes.
const RENDERS_AT_ONCE: usize = 2;

/// How many renditions this node asks the peers for at once.
const FETCHES_AT_ONCE: usize = 8;

/// The WebP quality of each rendition, out of 100: the large one is looked
/// at, the small one is small anyway.
const PAGE_QUALITY: f32 = 82.0;
const LISTING_QUALITY: f32 = 80.0;

/// Where the files are, and the nodes that keep them too.
pub struct Images {
    dir: PathBuf,
    /// The other nodes, when there are any.
    peers: Option<Peers>,
    renders: Semaphore,
    fetches: Semaphore,
}

/// Why what was given is not taken as a picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Not a JPEG, PNG, WebP or GIF, or one that cannot be decoded.
    NotAPicture,
    /// Over [`MAX_IMAGE_BYTES`].
    TooLarge,
    /// Over [`MAX_PICTURE_PIXELS`].
    TooManyPixels,
}

impl From<Refusal> for ApiError {
    fn from(refusal: Refusal) -> Self {
        let code = match refusal {
            Refusal::NotAPicture => UNSUPPORTED_IMAGE,
            Refusal::TooLarge => IMAGE_TOO_LARGE,
            Refusal::TooManyPixels => IMAGE_TOO_MANY_PIXELS,
        };
        Self::refused(StatusCode::UNPROCESSABLE_ENTITY, code)
    }
}

/// A picture rendered into what is kept of it, not yet stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    pub picture: Picture,
    listing: Bytes,
    page: Bytes,
}

impl Rendered {
    pub fn bytes(&self, rendition: Rendition) -> &Bytes {
        match rendition {
            Rendition::Listing => &self.listing,
            Rendition::Page => &self.page,
        }
    }
}

/// What asking the peers for a rendition came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fetch {
    /// This node had it already.
    Held,
    /// Taken from a peer and stored.
    Fetched,
    /// Every peer said it has no copy, or holds one that is not the file
    /// recorded, and none was out of reach.
    Gone,
    /// Some peer could not be asked (down, or not serving), so a copy may
    /// still be had later; why, for the log.
    Unreachable(String),
}

impl Images {
    pub fn new(dir: impl Into<PathBuf>, peers: Option<Peers>) -> Self {
        Self {
            dir: dir.into(),
            peers,
            renders: Semaphore::new(RENDERS_AT_ONCE),
            fetches: Semaphore::new(FETCHES_AT_ONCE),
        }
    }

    /// Where a rendition's file is, once its name is checked: one entry of
    /// the rendition's directory, nothing above. Everything that touches
    /// the disk goes through here.
    pub fn path_of(&self, rendition: Rendition, file_id: &str) -> anyhow::Result<PathBuf> {
        anyhow::ensure!(
            is_id(file_id),
            "{file_id:?} is not the id of a picture's files"
        );
        Ok(self
            .dir
            .join(rendition.dir())
            .join(format!("{file_id}.{RENDITION_EXT}")))
    }

    /// Renders what was given into the two renditions kept, off the async
    /// threads and a couple at a time. `Err` is a refusal to tell whoever
    /// gave it; the outer error is this node's.
    pub async fn render(&self, bytes: Bytes) -> anyhow::Result<Result<Rendered, Refusal>> {
        if bytes.len() > MAX_IMAGE_BYTES {
            return Ok(Err(Refusal::TooLarge));
        }
        let _slot = self.renders.acquire().await?;
        Ok(tokio::task::spawn_blocking(move || render(&bytes)).await?)
    }

    /// Writes a rendered picture to this node's disk under `file_id` and
    /// copies it to a majority of the cluster, like the event that will
    /// record it. Fails, with this node's files deleted again, when this
    /// node cannot write or too few nodes took the copies; the picture must
    /// then not be recorded. What it returns is what the event records.
    pub async fn keep(&self, file_id: &str, rendered: &Rendered) -> anyhow::Result<Picture> {
        let kept = async {
            for rendition in Rendition::ALL {
                let bytes = rendered.bytes(rendition).clone();
                self.store(rendition, file_id, bytes, Durability::Synced)
                    .await?;
            }
            let Some(peers) = &self.peers else {
                return Ok(());
            };
            let [listing, page] = Rendition::ALL.map(|rendition| async move {
                let path = format!("{PEER_ROUTE}/{}", Picture::file(file_id, rendition));
                let bytes = rendered.bytes(rendition).clone();
                peers.put_to_quorum(&path, bytes).await
            });
            let (listing, page) = tokio::join!(listing, page);
            listing.and(page)
        }
        .await;
        if let Err(err) = kept {
            self.remove(file_id).await;
            return Err(err);
        }
        Ok(rendered.picture)
    }

    /// Writes one rendition, whole from the moment it has its name; with
    /// [`Durability::Synced`] the name is on the disk too when this
    /// returns.
    pub async fn store(
        &self,
        rendition: Rendition,
        file_id: &str,
        bytes: Bytes,
        durability: Durability,
    ) -> anyhow::Result<()> {
        let path = self.path_of(rendition, file_id)?;
        tokio::task::spawn_blocking(move || disk::write(&path, &bytes, durability)).await??;
        Ok(())
    }

    /// One rendition as this node holds it; `None` when it holds no such
    /// file.
    pub async fn read(&self, rendition: Rendition, file_id: &str) -> anyhow::Result<Option<Bytes>> {
        let path = self.path_of(rendition, file_id)?;
        match tokio::fs::read(&path).await {
            Ok(bytes) => Ok(Some(bytes.into())),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(anyhow::anyhow!("cannot read {}: {err}", path.display())),
        }
    }

    /// Makes sure this node holds one rendition of a picture, asking the
    /// peers in turn for what it lacks; a copy is kept only if it is the
    /// size and has the digest recorded. A bounded number run at once. The
    /// error is this node's own; what the peers said is the [`Fetch`].
    pub async fn fetch_rendition(
        &self,
        file_id: &str,
        rendition: Rendition,
        recorded: &RenditionFile,
    ) -> anyhow::Result<Fetch> {
        let path = self.path_of(rendition, file_id)?;
        if tokio::fs::metadata(&path)
            .await
            .is_ok_and(|meta| meta.len() == recorded.size)
        {
            return Ok(Fetch::Held);
        }
        let Some(peers) = &self.peers else {
            // Alone, there is nobody to ask.
            return Ok(Fetch::Gone);
        };
        let file = Picture::file(file_id, rendition);
        let _slot = self.fetches.acquire().await?;
        let mut unreachable = None;
        for url in peers.urls() {
            let response = peers
                .client()
                .get(format!("{url}{PEER_ROUTE}/{file}"))
                .send()
                .await;
            let response = match response {
                Ok(response) if response.status() == reqwest::StatusCode::NOT_FOUND => continue,
                Ok(response) if response.status().is_success() => response,
                // The peer is up but not serving: not an answer about the
                // file.
                Ok(response) => {
                    unreachable
                        .get_or_insert_with(|| format!("{url} answered {}", response.status()));
                    continue;
                }
                Err(err) => {
                    unreachable.get_or_insert_with(|| format!("{url}: {err}"));
                    continue;
                }
            };
            let bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => {
                    unreachable.get_or_insert_with(|| format!("{url}: {err}"));
                    continue;
                }
            };
            if let Err(err) = check_copy(&bytes, recorded) {
                tracing::warn!(peer = %url, %file, %err, "a peer's copy is not the picture recorded");
                continue;
            }
            self.store(rendition, file_id, bytes, Durability::Cached)
                .await?;
            tracing::info!(peer = %url, %file, "fetched a picture from a peer");
            return Ok(Fetch::Fetched);
        }
        Ok(match unreachable {
            Some(reason) => Fetch::Unreachable(reason),
            None => Fetch::Gone,
        })
    }

    /// Takes a copy a peer sends: checks its name cannot leave the
    /// directory and that the bytes are a rendition (WebP, no heavier than
    /// one gets), then stores it. `false` when it is refused.
    pub async fn accept(
        &self,
        rendition: Rendition,
        file_id: &str,
        bytes: Bytes,
    ) -> anyhow::Result<bool> {
        if !is_id(file_id) || bytes.len() > MAX_RENDITION_BYTES || !is_rendition(&bytes) {
            return Ok(false);
        }
        self.store(rendition, file_id, bytes, Durability::Synced)
            .await?;
        Ok(true)
    }

    /// Deletes every rendition of a picture from this node. A failure is
    /// logged and otherwise ignored: nothing points at the files any more,
    /// and a file nobody shows is a waste of disk, not a fault.
    pub async fn remove(&self, file_id: &str) {
        for rendition in Rendition::ALL {
            let path = match self.path_of(rendition, file_id) {
                Ok(path) => path,
                Err(err) => {
                    tracing::warn!(%err, "not a picture of ours: not deleted");
                    return;
                }
            };
            if let Err(err) = tokio::fs::remove_file(&path).await
                && err.kind() != std::io::ErrorKind::NotFound
            {
                tracing::warn!(path = %path.display(), %err, "cannot delete a picture");
            }
        }
    }
}

/// The size and the files' id an address names: `<400|1200>` then
/// `<file id>.webp`. `None` for anything else.
pub fn parse_file(dir: &str, file: &str) -> Option<(Rendition, String)> {
    let rendition = Rendition::from_dir(dir)?;
    let file_id = file.strip_suffix(&format!(".{RENDITION_EXT}"))?;
    is_id(file_id).then(|| (rendition, file_id.to_owned()))
}

/// Whether `id` may name a picture's files: letters and digits, as the ids
/// drawn for them are, so it names one entry of a directory and nothing
/// above it.
fn is_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

/// The digest a picture is known by: the SHA-256 of the bytes it was given
/// as, in lowercase hex.
pub fn digest(bytes: &[u8]) -> String {
    sha256(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Whether a copy is the file the store recorded: its size and digest.
fn check_copy(bytes: &[u8], recorded: &RenditionFile) -> anyhow::Result<()> {
    anyhow::ensure!(
        bytes.len() as u64 == recorded.size,
        "got {} bytes, expected {}",
        bytes.len(),
        recorded.size
    );
    anyhow::ensure!(
        sha256(bytes) == recorded.sha256,
        "the bytes are not the ones recorded"
    );
    Ok(())
}

/// Whether the bytes are a WebP picture, as every rendition is.
fn is_rendition(bytes: &[u8]) -> bool {
    matches!(imagesize::image_type(bytes), Ok(imagesize::ImageType::Webp))
}

/// The width and height the header of `bytes` says, when they are a
/// picture of a kind that is taken. Only the header is read, so an enormous
/// file costs nothing to refuse.
fn sniff(bytes: &[u8]) -> Option<(u32, u32)> {
    use imagesize::ImageType::{Gif, Jpeg, Png, Webp};
    if !matches!(imagesize::image_type(bytes).ok()?, Jpeg | Png | Webp | Gif) {
        return None;
    }
    let size = imagesize::blob_size(bytes).ok()?;
    Some((size.width.try_into().ok()?, size.height.try_into().ok()?))
}

/// Renders a picture: decodes it (JPEG, PNG, WebP or GIF, the first frame
/// of an animation, turned the way its EXIF orientation says), bounds it to
/// the large rendition's side and then to the small one's, never enlarging,
/// and encodes each as WebP. Blocking: a second or so for a 25-megapixel
/// photograph.
fn render(bytes: &[u8]) -> Result<Rendered, Refusal> {
    // The header first: a picture with too many pixels is refused before a
    // byte of it is decoded.
    let (width, height) = sniff(bytes).ok_or(Refusal::NotAPicture)?;
    if width == 0 || height == 0 {
        return Err(Refusal::NotAPicture);
    }
    if u64::from(width) * u64::from(height) > MAX_PICTURE_PIXELS {
        return Err(Refusal::TooManyPixels);
    }
    let mut decoder = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| Refusal::NotAPicture)?
        .into_decoder()
        .map_err(|_| Refusal::NotAPicture)?;
    // What the header said is what gets decoded, and no more.
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(width);
    limits.max_image_height = Some(height);
    limits.max_alloc = Some(MAX_PICTURE_PIXELS * 8);
    decoder
        .set_limits(limits)
        .map_err(|_| Refusal::TooManyPixels)?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut given = DynamicImage::from_decoder(decoder).map_err(|_| Refusal::NotAPicture)?;
    given.apply_orientation(orientation);
    let (width, height) = given.dimensions();
    if width == 0 || height == 0 {
        return Err(Refusal::NotAPicture);
    }

    let page_image = fit(&given, Rendition::Page);
    let listing_image = fit(&page_image, Rendition::Listing);
    let (page, page_file) = encode(&page_image, PAGE_QUALITY)?;
    let (listing, listing_file) = encode(&listing_image, LISTING_QUALITY)?;
    Ok(Rendered {
        picture: Picture {
            width,
            height,
            listing: listing_file,
            page: page_file,
        },
        listing,
        page,
    })
}

/// The picture within the rendition's side, shrunk if it is larger and left
/// alone otherwise.
fn fit(image: &DynamicImage, rendition: Rendition) -> DynamicImage {
    let side = rendition.max_side();
    if image.width() <= side && image.height() <= side {
        return image.clone();
    }
    image.resize(side, side, FilterType::Lanczos3)
}

/// The picture as WebP, with what the store records of the file.
fn encode(image: &DynamicImage, quality: f32) -> Result<(Bytes, RenditionFile), Refusal> {
    let (width, height) = image.dimensions();
    let encoded = if image.color().has_alpha() {
        let rgba = image.to_rgba8();
        webp::Encoder::from_rgba(rgba.as_raw(), width, height).encode(quality)
    } else {
        let rgb = image.to_rgb8();
        webp::Encoder::from_rgb(rgb.as_raw(), width, height).encode(quality)
    };
    let bytes = Bytes::copy_from_slice(&encoded);
    if bytes.is_empty() || bytes.len() > MAX_RENDITION_BYTES {
        return Err(Refusal::NotAPicture);
    }
    let file = RenditionFile {
        width,
        height,
        size: bytes.len() as u64,
        sha256: sha256(&bytes),
    };
    Ok((bytes, file))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PNG of `width` × `height` in one colour.
    fn png(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
        let image = image::RgbaImage::from_pixel(width, height, image::Rgba(rgba));
        let mut out = Cursor::new(Vec::new());
        image.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    fn images() -> (Images, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        (Images::new(dir.path().join("images"), None), dir)
    }

    /// A picture becomes two WebP files, each within its side and never
    /// enlarged, and the record says what it and the files were.
    #[test]
    fn a_picture_is_rendered_in_two_sizes() {
        let wide = render(&png(3000, 1000, [200, 30, 30, 255])).unwrap();
        assert_eq!((wide.picture.width, wide.picture.height), (3000, 1000));
        let (page, listing) = (wide.picture.page, wide.picture.listing);
        assert_eq!((page.width, page.height), (1200, 400));
        assert_eq!((listing.width, listing.height), (400, 133));
        for (rendition, file) in wide.picture.files() {
            let bytes = wide.bytes(rendition);
            assert_eq!(bytes.len() as u64, file.size);
            assert_eq!(sha256(bytes), file.sha256);
            assert!(is_rendition(bytes), "{rendition:?} is WebP");
            let decoded = image::load_from_memory(bytes).unwrap();
            assert_eq!(decoded.dimensions(), (file.width, file.height));
        }
        assert!(wide.picture.is_valid());

        let small = render(&png(50, 80, [0, 0, 0, 128])).unwrap();
        let (page, listing) = (small.picture.page, small.picture.listing);
        assert_eq!((page.width, page.height), (50, 80));
        assert_eq!((listing.width, listing.height), (50, 80), "never enlarged");
        // Its transparency survives.
        let decoded = image::load_from_memory(&small.page).unwrap();
        assert!(decoded.color().has_alpha());

        // The same bytes render the same files, whoever gives them.
        assert_eq!(render(&png(50, 80, [0, 0, 0, 128])).unwrap(), small);
    }

    /// What is not a picture, or more picture than is taken, is refused
    /// with the word for it, before it is decoded.
    #[tokio::test]
    async fn what_is_not_a_picture_is_refused() {
        assert_eq!(render(b"not a picture"), Err(Refusal::NotAPicture));
        assert_eq!(render(&[]), Err(Refusal::NotAPicture));
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#;
        assert_eq!(render(svg), Err(Refusal::NotAPicture));
        // A PNG header claiming 6000 × 6000 pixels, and nothing behind it.
        let mut huge = png(1, 1, [0; 4]);
        huge[16..20].copy_from_slice(&6000u32.to_be_bytes());
        huge[20..24].copy_from_slice(&6000u32.to_be_bytes());
        assert_eq!(render(&huge), Err(Refusal::TooManyPixels));
        // A header that lies about a size within the cap is found out.
        let mut lying = png(1, 1, [0; 4]);
        lying[16..20].copy_from_slice(&600u32.to_be_bytes());
        assert!(render(&lying).is_err());

        let (images, _dir) = images();
        let heavy = Bytes::from(vec![0; MAX_IMAGE_BYTES + 1]);
        assert_eq!(images.render(heavy).await.unwrap(), Err(Refusal::TooLarge));
    }

    #[tokio::test]
    async fn a_node_alone_keeps_reads_and_removes_a_picture() {
        let (images, _dir) = images();
        let rendered = render(&png(900, 600, [10, 200, 10, 255])).unwrap();

        let picture = images.keep("01FILE", &rendered).await.unwrap();
        assert_eq!(picture, rendered.picture);
        for (rendition, file) in picture.files() {
            let held = images.read(rendition, "01FILE").await.unwrap().unwrap();
            assert_eq!(&held, rendered.bytes(rendition));
            let fetch = images.fetch_rendition("01FILE", rendition, file).await;
            assert_eq!(fetch.unwrap(), Fetch::Held);
        }
        // What it lacks, a node alone has nobody to ask for.
        let fetch = images
            .fetch_rendition("01OTHER", Rendition::Page, &picture.page)
            .await;
        assert_eq!(fetch.unwrap(), Fetch::Gone);

        images.remove("01FILE").await;
        assert_eq!(images.read(Rendition::Page, "01FILE").await.unwrap(), None);
    }

    /// The names come from events and addresses: none of them, whatever it
    /// says, gets a file written, read or deleted outside the directory.
    #[tokio::test]
    async fn a_name_cannot_leave_the_directory() {
        let (images, dir) = images();
        let rendered = render(&png(20, 20, [1, 2, 3, 255])).unwrap();
        let webp = rendered.listing.clone();

        for name in ["..", "../x", "a/b", "a.b", "", &"x".repeat(65)] {
            assert!(images.path_of(Rendition::Page, name).is_err(), "{name}");
            assert!(images.keep(name, &rendered).await.is_err(), "{name}");
            assert!(images.read(Rendition::Page, name).await.is_err(), "{name}");
            let accepted = images.accept(Rendition::Page, name, webp.clone()).await;
            assert!(!accepted.unwrap(), "{name}");
        }
        assert_eq!(
            parse_file("400", "01FILE.webp"),
            Some((Rendition::Listing, "01FILE".to_owned()))
        );
        for (size, file) in [
            ("800", "01FILE.webp"),
            ("400", "01FILE.png"),
            ("400", "../01FILE.webp"),
            ("..", "01FILE.webp"),
            ("400", ".webp"),
        ] {
            assert_eq!(parse_file(size, file), None, "{size}/{file}");
        }

        // A file above the directory survives every way of asking for its
        // deletion.
        let above = dir.path().join("x.webp");
        std::fs::write(&above, &webp).unwrap();
        images.remove("../x").await;
        images.remove("..").await;
        assert!(above.is_file());

        // A copy must be a rendition: WebP, and no heavier than one gets.
        let png = Bytes::from(png(2, 2, [0; 4]));
        assert!(!images.accept(Rendition::Page, "01A", png).await.unwrap());
        let heavy = Bytes::from(vec![0; MAX_RENDITION_BYTES + 1]);
        assert!(!images.accept(Rendition::Page, "01A", heavy).await.unwrap());
        assert!(images.accept(Rendition::Page, "01A", webp).await.unwrap());
    }
}
