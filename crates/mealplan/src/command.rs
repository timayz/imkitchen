use evento::{AggregatorName, Executor, LoadResult, SubscribeBuilder};
use imkitchen_db::table::MealPlanRecipe;
use imkitchen_recipe::{
    AdvancePrepChanged, BasicInformationChanged, Created, CuisineTypeChanged, Deleted,
    DietaryRestrictionsChanged, Imported, Ingredient, IngredientsChanged, Instruction,
    InstructionsChanged, MadePrivate, MainCourseOptionsChanged, Recipe, RecipeType,
    RecipeTypeChanged, SharedToCommunity,
};
use imkitchen_shared::{Event, Metadata};
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

use crate::{GenerateRequested, MealPlan, Status, WeekGenerated};

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
    ) -> Result<LoadResult<MealPlan>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn load_optional(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<MealPlan>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }

    pub async fn generate(&self, metadata: &Metadata) -> imkitchen_shared::Result<String> {
        let user_id = metadata.trigger_by()?;

        Ok(evento::save::<MealPlan>(user_id)
            .data(&GenerateRequested {
                status: Status::Processing,
            })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?)
    }
}

pub fn subscribe_command<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("mealplan-command")
        .handler(handle_generation_requested())
        .skip::<MealPlan, WeekGenerated>()
        .handler(handle_recipe_created())
        .handler(handle_recipe_imported())
        .handler(handle_recipe_type_changed())
        .handler(handle_recipe_basic_information_changed())
        .handler(handle_recipe_ingredients_changed())
        .handler(handle_recipe_instructions_changed())
        .handler(handle_recipe_advance_prep_changed())
        .handler(handle_recipe_made_private())
        .handler(handle_recipe_deleted())
        .skip::<Recipe, MainCourseOptionsChanged>()
        .skip::<Recipe, SharedToCommunity>()
        .skip::<Recipe, DietaryRestrictionsChanged>()
        .skip::<Recipe, CuisineTypeChanged>()
}

#[evento::handler(MealPlan)]
async fn handle_generation_requested<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<GenerateRequested>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    // let timestamp = event.timestamp;
    // let aggregator_id = event.aggregator_id.clone();
    // let user_id = event.metadata.trigger_by().unwrap_or_default();
    // let name = event.data.name;
    // let config = bincode::config::standard();
    // let instructions = bincode::encode_to_vec(Vec::<Instruction>::default(), config)?;
    // let ingredients = bincode::encode_to_vec(Vec::<Ingredient>::default(), config)?;
    //
    // let statment = Query::insert()
    //     .into_table(MealPlanRecipe::Table)
    //     .columns([
    //         MealPlanRecipe::Id,
    //         MealPlanRecipe::UserId,
    //         MealPlanRecipe::RecipeType,
    //         MealPlanRecipe::CuisineType,
    //         MealPlanRecipe::Name,
    //         MealPlanRecipe::Ingredients,
    //         MealPlanRecipe::Instructions,
    //         MealPlanRecipe::DietaryRestrictions,
    //         MealPlanRecipe::PreferredAccompanimentTypes,
    //         MealPlanRecipe::CreatedAt,
    //     ])
    //     .values_panic([
    //         aggregator_id.into(),
    //         user_id.into(),
    //         RecipeType::default().to_string().into(),
    //         CuisineType::default().to_string().into(),
    //         name.into(),
    //         ingredients.into(),
    //         instructions.into(),
    //         serde_json::Value::Array(vec![]).into(),
    //         serde_json::Value::Array(vec![]).into(),
    //         timestamp.into(),
    //     ])
    //     .to_owned();
    // let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    // sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_created<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let aggregator_id = event.aggregator_id.clone();
    let user_id = event.metadata.trigger_by().unwrap_or_default();
    let name = event.data.name;
    let config = bincode::config::standard();
    let instructions = bincode::encode_to_vec(Vec::<Instruction>::default(), config)?;
    let ingredients = bincode::encode_to_vec(Vec::<Ingredient>::default(), config)?;

    let statment = Query::insert()
        .into_table(MealPlanRecipe::Table)
        .columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::UserId,
            MealPlanRecipe::RecipeType,
            MealPlanRecipe::Name,
            MealPlanRecipe::Ingredients,
            MealPlanRecipe::Instructions,
        ])
        .values_panic([
            aggregator_id.into(),
            user_id.into(),
            RecipeType::default().to_string().into(),
            name.into(),
            ingredients.into(),
            instructions.into(),
        ])
        .to_owned();
    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_imported<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Imported>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let aggregator_id = event.aggregator_id.to_owned();
    let user_id = event.metadata.trigger_by()?;
    let name = event.data.name;
    let config = bincode::config::standard();
    let instructions = bincode::encode_to_vec(event.data.instructions, config)?;
    let ingredients = bincode::encode_to_vec(event.data.ingredients, config)?;

    let statment = Query::insert()
        .into_table(MealPlanRecipe::Table)
        .columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::UserId,
            MealPlanRecipe::Name,
            MealPlanRecipe::RecipeType,
            MealPlanRecipe::PrepTime,
            MealPlanRecipe::CookTime,
            MealPlanRecipe::Ingredients,
            MealPlanRecipe::Instructions,
            MealPlanRecipe::AdvancePrep,
        ])
        .values_panic([
            aggregator_id.into(),
            user_id.into(),
            name.into(),
            event.data.recipe_type.to_string().into(),
            event.data.prep_time.into(),
            event.data.cook_time.into(),
            ingredients.into(),
            instructions.into(),
            event.data.advance_prep.into(),
        ])
        .to_owned();
    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_type_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<RecipeTypeChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.trigger_by()?;
    let statment = Query::update()
        .table(MealPlanRecipe::Table)
        .values([(
            MealPlanRecipe::RecipeType,
            event.data.recipe_type.to_string().into(),
        )])
        .and_where(Expr::col(MealPlanRecipe::Id).eq(&event.aggregator_id))
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_basic_information_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<BasicInformationChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.trigger_by()?;
    let aggregator_id = event.aggregator_id.clone();
    let name = event.data.name;
    let prep_time = event.data.prep_time;
    let cook_time = event.data.cook_time;

    let statment = Query::update()
        .table(MealPlanRecipe::Table)
        .values([
            (MealPlanRecipe::Name, name.into()),
            (MealPlanRecipe::PrepTime, prep_time.into()),
            (MealPlanRecipe::CookTime, cook_time.into()),
        ])
        .and_where(Expr::col(MealPlanRecipe::Id).eq(aggregator_id))
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_ingredients_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<IngredientsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let ingredients = bincode::encode_to_vec(&event.data.ingredients, config)?;
    let user_id = event.metadata.trigger_by()?;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(MealPlanRecipe::Table)
        .values([(MealPlanRecipe::Ingredients, ingredients.into())])
        .and_where(Expr::col(MealPlanRecipe::Id).eq(aggregator_id))
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_instructions_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<InstructionsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let instructions = bincode::encode_to_vec(&event.data.instructions, config)?;
    let user_id = event.metadata.trigger_by()?;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(MealPlanRecipe::Table)
        .values([(MealPlanRecipe::Instructions, instructions.into())])
        .and_where(Expr::col(MealPlanRecipe::Id).eq(aggregator_id))
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_advance_prep_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<AdvancePrepChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.trigger_by()?;
    let aggregator_id = event.aggregator_id.clone();
    let description = event.data.advance_prep;

    let statment = Query::update()
        .table(MealPlanRecipe::Table)
        .values([(MealPlanRecipe::AdvancePrep, description.into())])
        .and_where(Expr::col(MealPlanRecipe::Id).eq(aggregator_id))
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_made_private<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MadePrivate>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let user_id = event.metadata.trigger_by()?;
    let statment = Query::delete()
        .from_table(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::Id).eq(&event.aggregator_id))
        .and_where(Expr::col(MealPlanRecipe::UserId).not_equals(user_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_deleted<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::delete()
        .from_table(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
