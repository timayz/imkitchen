//! The pictures of the recipes, each stored once however many recipes show
//! it: two users who import the same recipe share its picture.
//!
//! A picture is known by what it is: its stream is under the digest of the
//! bytes it was given as (`image_id`). The stream says which files hold it
//! on the nodes and who uses it. Each use has an id of its own, written
//! here before the recipe names it and released once the recipe let go of
//! it; the files are kept as long as one use is left.

use bitcode::{Decode, Encode};
use timada_core::Picture;

/// One picture, under the SHA-256 of the bytes it was given as, in
/// lowercase hex.
#[evento::aggregate(name = "imkitchen/Image")]
pub enum Image {
    /// Its files are kept on the nodes under `file_id`, for a first use:
    /// the picture was never seen, or its last use was released and its
    /// files went with it. Files are never named twice: a picture stored
    /// again has a fresh `file_id`.
    Stored {
        file_id: String,
        /// What the files are, for a node to check a copy against.
        picture: Picture,
        use_id: String,
    },
    /// Used once more, its files as they are.
    Used { use_id: String },
    /// A use is over. The last one takes the files with it.
    Released { use_id: String },
}

/// The picture a recipe shows: one use of an [`Image`]. Persisted in
/// events: its fields are frozen as the variants are.
#[derive(Debug, Clone, PartialEq, Eq, Default, Encode, Decode)]
pub struct RecipeImage {
    /// The stream of the picture.
    pub image_id: String,
    /// The use of it that is this recipe's, to release when the recipe lets
    /// go of it.
    pub use_id: String,
    /// What its files are named after, the same for as long as the use
    /// lasts.
    pub file_id: String,
    pub picture: Picture,
}
