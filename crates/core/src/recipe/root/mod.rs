use bitcode::{Decode, Encode};
use evento::{
    AggregateExt, Executor, Projection, ProjectionAggregate,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use image::imageops::FilterType;
use imkitchen_db::recipe_thumbnail::RecipeThumbnail;
use imkitchen_types::recipe::{
    self, AdvancePrepChanged, BasicInformationChanged, Created, CuisineTypeChanged, Deleted,
    DietaryRestrictionsChanged, Imported, IngredientsChanged, InstructionsChanged, MadePrivate,
    MainCourseOptionsChanged, RecipeType, RecipeTypeChanged, SharedToCommunity, ThumbnailResized,
    ThumbnailUploaded,
};
use imkitchen_types::recipe_share::{self, AllMadePrivate, AllSharedToCommunity};
use sea_query::{Expr, ExprTrait, OnConflict, Query as SeaQuery, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sha3::{Digest, Sha3_224};
use std::ops::Deref;
use webp::Encoder;

mod create;
mod delete;
mod import;
mod make_all_private;
mod make_private;
mod share_all_to_community;
mod share_to_community;
mod update;
mod upload_thumbnail;

pub use import::ImportInput;
pub use update::UpdateInput;

#[derive(Clone)]
pub struct Module<E: Executor> {
    state: crate::State<E>,
    pub favorite: crate::recipe::favorite::Module<E>,
}

impl<E: Executor> Deref for Module<E> {
    type Target = crate::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<E: Executor> Module<E> {
    pub fn new(state: crate::State<E>) -> Self
    where
        crate::State<E>: Clone,
    {
        Self {
            favorite: crate::recipe::favorite::Module(state.clone()),
            state,
        }
    }
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Recipe>> {
        create_projection().load(id).execute(&self.executor).await
    }

    pub async fn load_share(&self, user_id: impl Into<String>) -> anyhow::Result<RecipeShareState> {
        let user_id = user_id.into();

        Ok(create_share_projection()
            .load(&user_id)
            .execute(&self.executor)
            .await?
            .unwrap_or_else(|| RecipeShareState {
                id: user_id,
                cursor: Default::default(),
            }))
    }
}

#[evento::projection(Encode, Decode)]
pub struct Recipe {
    pub id: String,
    pub owner_id: String,
    pub recipe_type: RecipeType,
    pub basic_information_hash: Vec<u8>,
    pub ingredients_hash: Vec<u8>,
    pub instructions_hash: Vec<u8>,
    pub dietary_restrictions_hash: Vec<u8>,
    pub advance_prep_hash: Vec<u8>,
    pub accepts_accompaniment: bool,
    pub is_shared: bool,
}

#[evento::projection(Encode, Decode)]
pub struct RecipeShareState {
    pub id: String,
}

pub fn create_share_projection<E: Executor>() -> Projection<E, RecipeShareState> {
    Projection::new::<recipe_share::RecipeShare>()
        .handler(handle_all_shared_to_community())
        .handler(handle_all_made_private())
        .strict()
}

impl ProjectionAggregate for RecipeShareState {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_all_shared_to_community(
    event: Event<AllSharedToCommunity>,
    data: &mut RecipeShareState,
) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    Ok(())
}

#[evento::handler]
async fn handle_all_made_private(
    event: Event<AllMadePrivate>,
    data: &mut RecipeShareState,
) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    Ok(())
}

pub fn create_projection<E: Executor>() -> Projection<E, Recipe> {
    Projection::new::<recipe::Recipe>()
        .revision(2)
        .tombstone::<Deleted>()
        .handler(handle_created())
        .handler(handle_imported())
        .handler(handle_made_private())
        .handler(handle_ingredients_changed())
        .handler(handle_recipe_type_changed())
        .handler(handle_shared_to_community())
        .handler(handle_advance_prep_changed())
        .handler(handle_instructions_changed())
        .handler(handle_basic_information_changed())
        .handler(handle_main_course_options_changed())
        .handler(handle_dietary_restrictions_changed())
        .skip::<ThumbnailUploaded>()
        .skip::<ThumbnailResized>()
        .skip::<CuisineTypeChanged>()
        .strict()
}

impl ProjectionAggregate for Recipe {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_created(event: Event<Created>, data: &mut Recipe) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.owner_id = event.metadata.requested_by()?;

    Ok(())
}

#[evento::handler]
async fn handle_imported(event: Event<Imported>, data: &mut Recipe) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.owner_id = event.metadata.requested_by()?;
    data.recipe_type = event.data.recipe_type;

    let mut hasher = Sha3_224::default();
    hasher.update(event.data.name);
    hasher.update(event.data.origin.unwrap_or_default());
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

    let mut hasher = Sha3_224::default();

    for restriction in event.data.dietary_restrictions {
        hasher.update(restriction.to_string());
    }

    data.dietary_restrictions_hash = hasher.finalize()[..].to_vec();

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
    hasher.update(event.data.origin.unwrap_or_default());
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
    let (read_db, write_db) = context.extract::<(sqlx::SqlitePool, sqlx::SqlitePool)>();

    // Load the transient original stashed by the upload command. If it is
    // absent this is an idempotent replay (the original was already consumed
    // and deleted); the resized variants are authoritative in recipe_thumbnail,
    // so there is nothing to do.
    let Some(original) = load_original(&read_db, &event.aggregate_id).await? else {
        tracing::debug!(
            id = %event.aggregate_id,
            "recipe-command.handle_thumbnail_uploaded.original_absent"
        );
        return Ok(());
    };

    let img = match image::load_from_memory(&original) {
        Ok(img) => img,
        Err(err) => {
            tracing::warn!(error = ?err, "recipe-command.handle_thumbnail_uploaded.load_from_memory");
            // Drop the unusable original so it does not linger.
            delete_original(&write_db, &event.aggregate_id).await?;
            return Ok(());
        }
    };

    let original_version = context
        .executor
        .original_version::<ThumbnailResized>(&event.aggregate_id)
        .await?
        .expect("aggregator exist");
    let mut builder = evento::append(&event.aggregate_id)
        .original_version(original_version)
        .metadata_from(&event.metadata)
        .to_owned();

    for (name, width, quality) in IMAGE_VARIANTS {
        // Scope the encode so the non-Send WebPMemory/Encoder are dropped before
        // the await below; only the owned Vec<u8> crosses the await point.
        let webp: Vec<u8> = {
            let resized = img.resize(*width, u32::MAX, FilterType::Lanczos3);
            let rgba = resized.to_rgba8();
            let encoder = Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
            encoder.encode(*quality).to_vec() // 0.0 - 100.0
        };

        // Authoritative write of the variant bytes. recipe_thumbnail is now the
        // source of truth for images; the event carries no bytes.
        upsert_variant(&write_db, &event.aggregate_id, name, webp).await?;

        // Byte-free marker so the version/blur projections react.
        builder.event(&ThumbnailResized {
            device: name.to_string(),
        });
    }
    builder.commit(context.executor).await?;

    // Variants are persisted; drop the transient original.
    delete_original(&write_db, &event.aggregate_id).await?;
    Ok(())
}

async fn load_original(pool: &sqlx::SqlitePool, id: &str) -> anyhow::Result<Option<Vec<u8>>> {
    let statement = SeaQuery::select()
        .column(RecipeThumbnail::Data)
        .from(RecipeThumbnail::Table)
        .and_where(Expr::col(RecipeThumbnail::Id).eq(id))
        .and_where(Expr::col(RecipeThumbnail::Device).eq("original"))
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    Ok(
        sqlx::query_scalar_with::<_, Vec<u8>, _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_optional(pool)
            .await?,
    )
}

async fn upsert_variant(
    pool: &sqlx::SqlitePool,
    id: &str,
    device: &str,
    data: Vec<u8>,
) -> anyhow::Result<()> {
    let statement = SeaQuery::insert()
        .into_table(RecipeThumbnail::Table)
        .columns([
            RecipeThumbnail::Id,
            RecipeThumbnail::Device,
            RecipeThumbnail::Data,
        ])
        .values_panic([id.into(), device.into(), data.into()])
        .on_conflict(
            OnConflict::columns([RecipeThumbnail::Id, RecipeThumbnail::Device])
                .update_column(RecipeThumbnail::Data)
                .to_owned(),
        )
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await?;
    Ok(())
}

async fn delete_original(pool: &sqlx::SqlitePool, id: &str) -> anyhow::Result<()> {
    let statement = SeaQuery::delete()
        .from_table(RecipeThumbnail::Table)
        .and_where(Expr::col(RecipeThumbnail::Id).eq(id))
        .and_where(Expr::col(RecipeThumbnail::Device).eq("original"))
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await?;
    Ok(())
}
