mod save;
mod unsave;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregate, metadata::Event};
use imkitchen_types::favorite::{self};
use sha3::{Digest, Sha3_256};
use std::ops::Deref;

#[derive(Clone)]
pub struct Module<E: Executor>(pub(crate) crate::State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = crate::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    /// The favorite of `user_id` for recipe `id`: the stream keyed by
    /// `hash_ids([id, user_id])`, or, when that stream is empty, the one
    /// keyed by [`legacy_id`] (favorites saved before evento alpha.28 /
    /// imkitchen 1.9.1). Whichever stream is found is the one later writes
    /// go to, since `Favorite::id` carries the stream's aggregate id.
    pub async fn load(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Favorite> {
        let id = id.into();
        let user_id = user_id.into();

        if let Some(favorite) = create_projection::<E>()
            .load_ids(vec![id.clone(), user_id.clone()])
            .execute(&self.executor)
            .await?
        {
            return Ok(favorite);
        }

        if let Some(favorite) = create_projection::<E>()
            .load(legacy_id(&id, &user_id))
            .execute(&self.executor)
            .await?
        {
            return Ok(favorite);
        }

        Ok(Favorite {
            id: evento::hash_ids(vec![id, user_id]),
            saved: false,
            cursor: Default::default(),
            aggregate_version: Default::default(),
            tenant: Default::default(),
        })
    }
}

/// The aggregate id `evento::hash_ids([recipe_id, user_id])` produced before
/// evento 2.0.0-alpha.28 (imkitchen 1.9.1): SHA3-256 over the plain
/// concatenation of the ids, before the per-id length prefix was added.
/// Favorites saved until then still live in streams keyed this way.
pub fn legacy_id(recipe_id: &str, user_id: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(recipe_id);
    hasher.update(user_id);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[evento::projection(name = "imkitchen-core/recipe/favorite/Favorite", Encode, Decode)]
pub struct Favorite {
    pub id: String,
    pub saved: bool,
}

pub fn create_projection<E: Executor>() -> Projection<E, Favorite> {
    Projection::new::<favorite::Favorite>()
        .handler(handle_saved())
        .handler(handle_unsaved())
        .strict()
        // Bumped from the implicit 0 → 1 when evento's `#[projection]` macro
        // grew the `aggregate_version` field: invalidates old snapshots so they
        // rebuild from events rather than failing to bitcode-decode.
        .revision(1)
}

impl ProjectionAggregate for Favorite {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_saved(event: Event<favorite::Saved>, data: &mut Favorite) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.saved = true;

    Ok(())
}

#[evento::handler]
async fn handle_unsaved(
    event: Event<favorite::Unsaved>,
    data: &mut Favorite,
) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.saved = false;

    Ok(())
}
