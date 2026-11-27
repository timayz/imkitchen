//! The write side of a recipe: what its stream says of it, the commands of
//! its owner and the two an administrator has over anyone's ([`remove`],
//! [`remove_image_shown`]).
//!
//! The state folds every event of the stream (`strict()`), so the version
//! it holds, which a write goes out at, is never stale. It is not
//! snapshotted: a recipe is rewritten a few dozen times at most.
//!
//! A recipe names the use it makes of a picture (`crate::images`), written
//! on the picture's stream before the recipe is given it. What it let go of
//! is kept in the state ([`RecipeState::let_go`]) for the subscription that
//! gives those uses back.

use axum::http::StatusCode;
use evento::metadata::Event;
use evento::{Executor, Projection, ProjectionAggregate, WriteBuilder, WriteError};
use imkitchen_aggregates::image::RecipeImage;
use imkitchen_aggregates::recipe::{
    Created, Deleted, Edited, ImageRemoved, ImageSet, Ingredient, Instruction, Recipe,
};

use crate::error::{ApiError, ApiResult};

/// The recipe does not exist, was deleted, or is someone else's: all the
/// same to who asks.
pub const RECIPE_NOT_FOUND: &str = "recipe_not_found";

/// How many times a write that lost to another on the same recipe is made
/// again, over the recipe as the other left it.
const WRITE_ATTEMPTS: u32 = 3;

/// Everything a recipe says. Times are minutes, 0 for none.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Content {
    pub title: String,
    pub description: String,
    pub ingredients: Vec<Ingredient>,
    /// The steps, in the order they are made.
    pub instructions: Vec<Instruction>,
    pub prep_minutes: u32,
    pub cook_minutes: u32,
    pub servings: u32,
}

/// A recipe as its stream leaves it.
#[evento::projection(id = id, name = "imkitchen/RecipeState")]
#[evento::snapshot(none)]
#[derive(Debug, PartialEq)]
pub struct RecipeState {
    pub id: String,
    pub owner_id: String,
    pub content: Content,
    /// The picture it shows, when it shows one.
    pub image: Option<RecipeImage>,
    /// The pictures it showed and let go of: given another, shown without
    /// one, or deleted.
    pub let_go: Vec<RecipeImage>,
    /// Unix seconds of `Created`.
    pub created_at: u64,
    /// Unix seconds of the last event.
    pub updated_at: u64,
    pub deleted: bool,
}

fn projection<E: Executor>() -> Projection<E, RecipeState> {
    Projection::new::<Recipe>()
        .handler(on_created())
        .handler(on_edited())
        .handler(on_deleted())
        .handler(on_image_set())
        .handler(on_image_removed())
        .strict()
}

#[evento::handler]
async fn on_created(event: Event<Created>, recipe: &mut RecipeState) -> anyhow::Result<()> {
    let data = &event.data;
    recipe.id = event.aggregate_id.to_owned();
    recipe.owner_id = data.owner_id.to_owned();
    recipe.content = Content {
        title: data.title.to_owned(),
        description: data.description.to_owned(),
        ingredients: data.ingredients.to_owned(),
        instructions: data.instructions.to_owned(),
        prep_minutes: data.prep_minutes,
        cook_minutes: data.cook_minutes,
        servings: data.servings,
    };
    recipe.created_at = event.timestamp;
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_edited(event: Event<Edited>, recipe: &mut RecipeState) -> anyhow::Result<()> {
    let data = &event.data;
    recipe.content = Content {
        title: data.title.to_owned(),
        description: data.description.to_owned(),
        ingredients: data.ingredients.to_owned(),
        instructions: data.instructions.to_owned(),
        prep_minutes: data.prep_minutes,
        cook_minutes: data.cook_minutes,
        servings: data.servings,
    };
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_deleted(event: Event<Deleted>, recipe: &mut RecipeState) -> anyhow::Result<()> {
    recipe.deleted = true;
    recipe.let_go.extend(recipe.image.take());
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_image_set(event: Event<ImageSet>, recipe: &mut RecipeState) -> anyhow::Result<()> {
    let before = recipe.image.replace(event.data.image.to_owned());
    recipe.let_go.extend(before);
    recipe.updated_at = event.timestamp;
    Ok(())
}

#[evento::handler]
async fn on_image_removed(
    event: Event<ImageRemoved>,
    recipe: &mut RecipeState,
) -> anyhow::Result<()> {
    recipe.let_go.extend(recipe.image.take());
    recipe.updated_at = event.timestamp;
    Ok(())
}

/// The recipe `id` as its stream leaves it, deleted or not.
pub async fn load<E: Executor>(executor: &E, id: &str) -> anyhow::Result<Option<RecipeState>> {
    projection().load(id).execute(executor).await
}

/// The recipe `id` of `owner_id`. Anyone else's is refused the way one that
/// does not exist is: asking tells nothing of what others wrote.
pub async fn load_own<E: Executor>(
    executor: &E,
    owner_id: &str,
    id: &str,
) -> ApiResult<RecipeState> {
    load(executor, id)
        .await?
        .filter(|recipe| !recipe.deleted && recipe.owner_id == owner_id)
        .ok_or(ApiError::refused(StatusCode::NOT_FOUND, RECIPE_NOT_FOUND))
}

/// The recipe `id`, whoever's it is: an administrator's to ask for. One
/// that does not exist or was deleted is refused.
async fn load_live<E: Executor>(executor: &E, id: &str) -> ApiResult<RecipeState> {
    load(executor, id)
        .await?
        .filter(|recipe| !recipe.deleted)
        .ok_or(ApiError::refused(StatusCode::NOT_FOUND, RECIPE_NOT_FOUND))
}

/// Writes a recipe of `owner_id` under a fresh id, which it answers, shown
/// with `image` when there is one.
pub async fn create<E: Executor>(
    executor: &E,
    owner_id: &str,
    content: Content,
    image: Option<RecipeImage>,
) -> ApiResult<String> {
    let mut write = evento::create();
    write.event(&Created {
        owner_id: owner_id.to_owned(),
        title: content.title,
        description: content.description,
        ingredients: content.ingredients,
        instructions: content.instructions,
        prep_minutes: content.prep_minutes,
        cook_minutes: content.cook_minutes,
        servings: content.servings,
    });
    if let Some(image) = image {
        write.event(&ImageSet { image });
    }
    Ok(write.commit(executor).await?)
}

/// Rewrites the recipe `id` of `owner_id` as `content`; the same as it
/// stands writes nothing.
pub async fn edit<E: Executor>(
    executor: &E,
    owner_id: &str,
    id: &str,
    content: Content,
) -> ApiResult<()> {
    write(executor, Some(owner_id), id, |recipe, write| {
        if recipe.content == content {
            return false;
        }
        let content = content.clone();
        write.event(&Edited {
            title: content.title,
            description: content.description,
            ingredients: content.ingredients,
            instructions: content.instructions,
            prep_minutes: content.prep_minutes,
            cook_minutes: content.cook_minutes,
            servings: content.servings,
        });
        true
    })
    .await
}

/// Shows the recipe `id` of `owner_id` with `image`, in place of the picture
/// it had.
pub async fn set_image<E: Executor>(
    executor: &E,
    owner_id: &str,
    id: &str,
    image: RecipeImage,
) -> ApiResult<()> {
    write(executor, Some(owner_id), id, |_, write| {
        write.event(&ImageSet {
            image: image.clone(),
        });
        true
    })
    .await
}

/// Shows the recipe `id` of `owner_id` without a picture; one that has none
/// writes nothing.
pub async fn remove_image<E: Executor>(executor: &E, owner_id: &str, id: &str) -> ApiResult<()> {
    write(executor, Some(owner_id), id, |recipe, write| {
        if recipe.image.is_none() {
            return false;
        }
        write.event(&ImageRemoved);
        true
    })
    .await
}

/// Deletes the recipe `id` of `owner_id` for good.
pub async fn delete<E: Executor>(executor: &E, owner_id: &str, id: &str) -> ApiResult<()> {
    write(executor, Some(owner_id), id, |_, write| {
        write.event(&Deleted);
        true
    })
    .await
}

/// Deletes the recipe `id` for good, whoever's it is: an administrator's
/// to do.
pub async fn remove<E: Executor>(executor: &E, id: &str) -> ApiResult<()> {
    write(executor, None, id, |_, write| {
        write.event(&Deleted);
        true
    })
    .await
}

/// Shows the recipe `id` without a picture, whoever's it is, when the one
/// it shows is `image_id`: an administrator's to do. One that shows another
/// picture or none writes nothing.
pub async fn remove_image_shown<E: Executor>(
    executor: &E,
    id: &str,
    image_id: &str,
) -> ApiResult<()> {
    write(executor, None, id, |recipe, write| {
        let shown = recipe.image.as_ref();
        if shown.is_none_or(|image| image.image_id != image_id) {
            return false;
        }
        write.event(&ImageRemoved);
        true
    })
    .await
}

/// Appends what `change` adds to the recipe `id`, at the version it was
/// loaded at: one of `owner_id`'s, or anyone's without one. `change`
/// answers `false` when there is nothing to write. A write another one got
/// ahead of is made again over the recipe as it now stands.
async fn write<E: Executor>(
    executor: &E,
    owner_id: Option<&str>,
    id: &str,
    change: impl Fn(&RecipeState, &mut WriteBuilder) -> bool,
) -> ApiResult<()> {
    for _ in 0..WRITE_ATTEMPTS {
        let recipe = match owner_id {
            Some(owner_id) => load_own(executor, owner_id, id).await?,
            None => load_live(executor, id).await?,
        };
        let mut write = recipe.write()?;
        if !change(&recipe, &mut write) {
            return Ok(());
        }
        match write.commit(executor).await {
            Ok(_) => return Ok(()),
            Err(WriteError::InvalidOriginalVersion) => continue,
            Err(err) => return Err(err.into()),
        }
    }
    Err(ApiError::refused(
        StatusCode::SERVICE_UNAVAILABLE,
        timada_core::WRITE_NOT_TAKEN,
    ))
}
