use evento::{Executor, Projection, Snapshot, metadata::Event};
use imkitchen_db::table::RecipeCommand;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sha3::{Digest, Sha3_224};
use sqlx::{SqlitePool, prelude::FromRow};

use crate::{
    AdvancePrepChanged, BasicInformationChanged, Created, CuisineType, CuisineTypeChanged, Deleted,
    DietaryRestrictionsChanged, Imported, IngredientsChanged, InstructionsChanged, MadePrivate,
    MainCourseOptionsChanged, Recipe, RecipeType, RecipeTypeChanged, SharedToCommunity,
};

mod create;
mod delete;
mod import;
mod make_private;
mod share_to_community;
mod update;

pub use import::ImportInput;
pub use update::UpdateInput;

#[evento::command]
#[derive(FromRow)]
pub struct Command {
    pub owner_id: String,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub cuisine_type: sqlx::types::Text<CuisineType>,
    pub basic_information_hash: Vec<u8>,
    pub ingredients_hash: Vec<u8>,
    pub instructions_hash: Vec<u8>,
    pub dietary_restrictions_hash: Vec<u8>,
    pub advance_prep_hash: Vec<u8>,
    pub accepts_accompaniment: bool,
    pub is_shared: bool,
    pub is_deleted: bool,
}

pub fn create_projection(id: impl Into<String>) -> Projection<CommandData> {
    Projection::new::<Recipe>(id)
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
        .safety_check()
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<Command<'a, E>>, anyhow::Error> {
    let id = id.into();

    let Some(data) = create_projection(&id)
        .data(pool.clone())
        .execute(executor)
        .await?
    else {
        return Ok(None);
    };

    Ok(Some(Command::new(
        id,
        data.get_cursor_version()?,
        data,
        executor,
    )))
}

impl Snapshot for CommandData {}

// #[evento::snapshot]
// async fn restore(
//     context: &evento::context::RwContext,
//     id: String,
//     _aggregators: &std::collections::HashMap<String, String>,
// ) -> anyhow::Result<Option<CommandData>> {
//     let pool = context.extract::<SqlitePool>();
//     let statement = Query::select()
//         .columns([
//             RecipeCommand::OwnerId,
//             RecipeCommand::RecipeType,
//             RecipeCommand::CuisineType,
//             RecipeCommand::IsShared,
//             RecipeCommand::BasicInformationHash,
//             RecipeCommand::IngredientsHash,
//             RecipeCommand::InstructionsHash,
//             RecipeCommand::DietaryRestrictionsHash,
//             RecipeCommand::AdvancePrepHash,
//             RecipeCommand::AcceptsAccompaniment,
//             RecipeCommand::IsDeleted,
//         ])
//         .from(RecipeCommand::Table)
//         .and_where(Expr::col(RecipeCommand::Id).eq(id))
//         .to_owned();
//
//     let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
//
//     Ok(sqlx::query_as_with(&sql, values)
//         .fetch_optional(&pool)
//         .await?)
// }

#[evento::handler]
async fn handle_created(event: Event<Created>, data: &mut CommandData) -> anyhow::Result<()> {
    data.owner_id = event.metadata.user()?;

    Ok(())
}

#[evento::handler]
async fn handle_imported(event: Event<Imported>, data: &mut CommandData) -> anyhow::Result<()> {
    data.owner_id = event.metadata.user()?;
    data.recipe_type.0 = event.data.recipe_type;
    data.cuisine_type.0 = event.data.cuisine_type;

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
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.recipe_type.0 = event.data.recipe_type;

    Ok(())
}

#[evento::handler]
async fn handle_basic_information_changed(
    event: Event<BasicInformationChanged>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    let id = event.aggregator_id.to_owned();
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
    data: &mut CommandData,
) -> anyhow::Result<()> {
    let id = event.aggregator_id.to_owned();
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
    data: &mut CommandData,
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
    data: &mut CommandData,
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
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.cuisine_type.0 = event.data.cuisine_type;

    Ok(())
}

#[evento::handler]
async fn handle_main_course_options_changed(
    event: Event<MainCourseOptionsChanged>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.accepts_accompaniment = event.data.accepts_accompaniment;

    Ok(())
}

#[evento::handler]
async fn handle_advance_prep_changed(
    event: Event<AdvancePrepChanged>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    let mut hasher = Sha3_224::default();
    hasher.update(event.data.advance_prep);

    data.advance_prep_hash = hasher.finalize()[..].to_vec();

    Ok(())
}

#[evento::handler]
async fn handle_shared_to_community(
    _event: Event<SharedToCommunity>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.is_shared = true;

    Ok(())
}

#[evento::handler]
async fn handle_made_private(
    _event: Event<MadePrivate>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.is_shared = false;

    Ok(())
}

#[evento::handler]
async fn handle_deleted(event: Event<Deleted>, data: &mut CommandData) -> anyhow::Result<()> {
    data.is_deleted = true;

    Ok(())
}

async fn update(
    pool: &SqlitePool,
    id: impl Into<Expr>,
    col: RecipeCommand,
    value: impl Into<Expr>,
) -> anyhow::Result<()> {
    let statement = Query::update()
        .table(RecipeCommand::Table)
        .value(col, value)
        .and_where(Expr::col(RecipeCommand::Id).eq(id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}
