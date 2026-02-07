use bitcode::{Decode, Encode};
use evento::{
    AggregatorExecutor, Executor, Projection, ProjectionAggregator,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use image::imageops::FilterType;
use imkitchen_shared::recipe::{
    self, AdvancePrepChanged, BasicInformationChanged, Created, CuisineType, CuisineTypeChanged,
    Deleted, DietaryRestrictionsChanged, Imported, IngredientsChanged, InstructionsChanged,
    MadePrivate, MainCourseOptionsChanged, RecipeType, RecipeTypeChanged, SharedToCommunity,
    ThumbnailResized, ThumbnailUploaded,
};
use sha3::{Digest, Sha3_224};
use std::ops::Deref;
use webp::Encoder;

mod create;
mod delete;
mod import;
mod make_private;
mod share_to_community;
mod update;
mod upload_thumbnail;

pub use import::ImportInput;
pub use update::UpdateInput;

#[derive(Clone)]
pub struct Command<E: Executor> {
    state: imkitchen_shared::State<E>,
    pub rating: crate::rating::Command<E>,
    pub favorite: crate::favorite::Command<E>,
    pub comment: crate::comment::Command<E>,
    pub comment_rating: crate::comment_rating::Command<E>,
}

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<E: Executor> Command<E> {
    pub fn new(state: imkitchen_shared::State<E>) -> Self
    where
        imkitchen_shared::State<E>: Clone,
    {
        Self {
            rating: crate::rating::Command(state.clone()),
            favorite: crate::favorite::Command(state.clone()),
            comment_rating: crate::comment_rating::Command(state.clone()),
            comment: crate::comment::Command(state.clone()),
            state,
        }
    }
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Recipe>> {
        let Some(recipe) = create_projection(id).execute(&self.executor).await? else {
            return Ok(None);
        };

        if recipe.is_deleted {
            return Ok(None);
        }

        Ok(Some(recipe))
    }
}

#[evento::projection(Encode, Decode)]
pub struct Recipe {
    pub id: String,
    pub owner_id: String,
    pub recipe_type: RecipeType,
    pub cuisine_type: CuisineType,
    pub basic_information_hash: Vec<u8>,
    pub ingredients_hash: Vec<u8>,
    pub instructions_hash: Vec<u8>,
    pub dietary_restrictions_hash: Vec<u8>,
    pub advance_prep_hash: Vec<u8>,
    pub accepts_accompaniment: bool,
    pub is_shared: bool,
    pub is_deleted: bool,
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, Recipe> {
    Projection::new::<recipe::Recipe>(id)
        .handler(handle_created())
        .handler(handle_deleted())
        .handler(handle_imported())
        .handler(handle_made_private())
        .handler(handle_ingredients_changed())
        .handler(handle_recipe_type_changed())
        .handler(handle_shared_to_community())
        .handler(handle_advance_prep_changed())
        .handler(handle_cuinine_type_changed())
        .handler(handle_instructions_changed())
        .handler(handle_basic_information_changed())
        .handler(handle_main_course_options_changed())
        .handler(handle_dietary_restrictions_changed())
        .skip::<ThumbnailUploaded>()
        .skip::<ThumbnailResized>()
        .safety_check()
}

impl ProjectionAggregator for Recipe {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_created(event: Event<Created>, data: &mut Recipe) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.owner_id = event.metadata.requested_by()?;

    Ok(())
}

#[evento::handler]
async fn handle_imported(event: Event<Imported>, data: &mut Recipe) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.owner_id = event.metadata.requested_by()?;
    data.recipe_type = event.data.recipe_type;
    data.cuisine_type = event.data.cuisine_type;

    let mut hasher = Sha3_224::default();
    hasher.update(event.data.name);
    hasher.update(event.data.description);
    hasher.update(event.data.household_size.to_string());
    hasher.update(event.data.prep_time.to_string());
    hasher.update(event.data.cook_time.to_string());

    data.basic_information_hash = hasher.finalize()[..].to_vec();

    let mut hasher = Sha3_224::default();

    for instruction in event.data.instructions {
        hasher.update(instruction.description);
        hasher.update(instruction.time_next.to_string());
    }

    data.instructions_hash = hasher.finalize()[..].to_vec();

    let mut hasher = Sha3_224::default();

    for ingredient in event.data.ingredients {
        hasher.update(ingredient.name);
        hasher.update(ingredient.quantity.to_string());

        if let Some(unit) = ingredient.unit {
            hasher.update(unit.to_string());
        }

        if let Some(category) = ingredient.category {
            hasher.update(category.to_string());
        }
    }

    data.ingredients_hash = hasher.finalize()[..].to_vec();

    let mut hasher = Sha3_224::default();
    hasher.update(event.data.advance_prep);

    data.advance_prep_hash = hasher.finalize()[..].to_vec();

    Ok(())
}

#[evento::handler]
async fn handle_recipe_type_changed(
    event: Event<RecipeTypeChanged>,
    data: &mut Recipe,
) -> anyhow::Result<()> {
    data.recipe_type = event.data.recipe_type;

    Ok(())
}

#[evento::handler]
async fn handle_basic_information_changed(
    event: Event<BasicInformationChanged>,
    data: &mut Recipe,
) -> anyhow::Result<()> {
    let mut hasher = Sha3_224::default();
    hasher.update(event.data.name);
    hasher.update(event.data.description);
    hasher.update(event.data.household_size.to_string());
    hasher.update(event.data.prep_time.to_string());
    hasher.update(event.data.cook_time.to_string());

    data.basic_information_hash = hasher.finalize()[..].to_vec();

    Ok(())
}

#[evento::handler]
async fn handle_instructions_changed(
    event: Event<InstructionsChanged>,
    data: &mut Recipe,
) -> anyhow::Result<()> {
    let mut hasher = Sha3_224::default();

    for instruction in event.data.instructions {
        hasher.update(instruction.description);
        hasher.update(instruction.time_next.to_string());
    }

    data.instructions_hash = hasher.finalize()[..].to_vec();

    Ok(())
}

#[evento::handler]
async fn handle_ingredients_changed(
    event: Event<IngredientsChanged>,
    data: &mut Recipe,
) -> anyhow::Result<()> {
    let mut hasher = Sha3_224::default();

    for ingredient in event.data.ingredients {
        hasher.update(ingredient.name);
        hasher.update(ingredient.quantity.to_string());

        if let Some(unit) = ingredient.unit {
            hasher.update(unit.to_string());
        }

        if let Some(category) = ingredient.category {
            hasher.update(category.to_string());
        }
    }

    data.ingredients_hash = hasher.finalize()[..].to_vec();

    Ok(())
}

#[evento::handler]
async fn handle_dietary_restrictions_changed(
    event: Event<DietaryRestrictionsChanged>,
    data: &mut Recipe,
) -> anyhow::Result<()> {
    let mut hasher = Sha3_224::default();

    for restriction in event.data.dietary_restrictions {
        hasher.update(restriction.to_string());
    }

    data.dietary_restrictions_hash = hasher.finalize()[..].to_vec();

    Ok(())
}

#[evento::handler]
async fn handle_cuinine_type_changed(
    event: Event<CuisineTypeChanged>,
    data: &mut Recipe,
) -> anyhow::Result<()> {
    data.cuisine_type = event.data.cuisine_type;

    Ok(())
}

#[evento::handler]
async fn handle_main_course_options_changed(
    event: Event<MainCourseOptionsChanged>,
    data: &mut Recipe,
) -> anyhow::Result<()> {
    data.accepts_accompaniment = event.data.accepts_accompaniment;

    Ok(())
}

#[evento::handler]
async fn handle_advance_prep_changed(
    event: Event<AdvancePrepChanged>,
    data: &mut Recipe,
) -> anyhow::Result<()> {
    let mut hasher = Sha3_224::default();
    hasher.update(event.data.advance_prep);

    data.advance_prep_hash = hasher.finalize()[..].to_vec();

    Ok(())
}

#[evento::handler]
async fn handle_shared_to_community(
    _event: Event<SharedToCommunity>,
    data: &mut Recipe,
) -> anyhow::Result<()> {
    data.is_shared = true;

    Ok(())
}

#[evento::handler]
async fn handle_made_private(_event: Event<MadePrivate>, data: &mut Recipe) -> anyhow::Result<()> {
    data.is_shared = false;

    Ok(())
}

#[evento::handler]
async fn handle_deleted(_event: Event<Deleted>, data: &mut Recipe) -> anyhow::Result<()> {
    data.is_deleted = true;

    Ok(())
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-command").handler(handle_thumbnail_uploaded())
}

const IMAGE_VARIANTS: &[(&str, u32, f32)] = &[
    //  name      width  quality
    ("mobile", 480, 60.0), // lower quality for mobile
    ("tablet", 768, 75.0),
    ("desktop", 1280, 85.0),
];

#[evento::subscription]
async fn handle_thumbnail_uploaded<E: Executor>(
    context: &Context<'_, E>,
    event: Event<ThumbnailUploaded>,
) -> anyhow::Result<()> {
    let original_version = context
        .executor
        .original_version::<ThumbnailResized>(&event.aggregator_id)
        .await?
        .expect("aggregator exist");
    let img = image::load_from_memory(&event.data.data)?;
    let mut builder = evento::aggregator(&event.aggregator_id)
        .original_version(original_version)
        .metadata_from(&event.metadata)
        .to_owned();

    for (name, width, quality) in IMAGE_VARIANTS {
        let resized = img.resize(*width, u32::MAX, FilterType::Lanczos3);
        let rgba = resized.to_rgba8();

        let encoder = Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());

        let webp = encoder.encode(*quality); // 0.0 - 100.0
        builder.event(&ThumbnailResized {
            device: name.to_string(),
            data: webp.to_vec(),
        });
    }
    builder.commit(context.executor).await?;
    Ok(())
}
