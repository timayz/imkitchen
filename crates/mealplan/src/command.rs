use evento::{AggregatorName, Executor, LoadResult, SubscribeBuilder};
use imkitchen_db::table::{MealPlanLastWeek, MealPlanRecipe};
use imkitchen_recipe::{
    AdvancePrepChanged, BasicInformationChanged, Created, CuisineTypeChanged, Deleted,
    DietaryRestrictionsChanged, Imported, Ingredient, IngredientsChanged, Instruction,
    InstructionsChanged, MadePrivate, MainCourseOptionsChanged, Recipe, RecipeType,
    RecipeTypeChanged, SharedToCommunity,
};
use imkitchen_shared::{Event, Metadata};
use rand::seq::SliceRandom;
use sea_query::{
    Expr, ExprTrait, Func, IntoColumnRef, OnConflict, Order, Query, SimpleExpr, SqliteQueryBuilder,
};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;
use time::{Duration, OffsetDateTime};

use crate::{GenerateRequested, GenerationFailed, MealPlan, Slot, Status, WeekGenerated};

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

    pub async fn generate(&self, metadata: &Metadata) -> imkitchen_shared::Result<()> {
        let user_id = metadata.trigger_by()?;

        let loaded = self.load_optional(&user_id).await?;
        let processing = loaded
            .as_ref()
            .map(|m| m.item.status == Status::Processing)
            .unwrap_or_default();
        if processing {
            imkitchen_shared::bail!("Meal plan status is processing");
        }

        let builder = loaded
            .map(evento::save_with)
            .unwrap_or_else(|| evento::save(&user_id));

        let weeks = super::service::next_four_mondays_from_now()
            .to_vec()
            .iter()
            .map(|week| {
                (
                    week.start.unix_timestamp() as u64,
                    week.end.unix_timestamp() as u64,
                )
            })
            .collect();

        builder
            .data(&GenerateRequested {
                weeks,
                status: Status::Processing,
            })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}

async fn has(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
    recipe_type: RecipeType,
) -> imkitchen_shared::Result<bool> {
    let id = id.into();
    let statement = Query::select()
        .columns([MealPlanRecipe::Id])
        .from(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(id))
        .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()))
        .and_where(Expr::col(MealPlanRecipe::Name).is_not(""))
        .limit(1)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let recipe = sqlx::query_as_with::<_, (String,), _>(&sql, values)
        .fetch_optional(pool)
        .await?;

    Ok(recipe.is_some())
}

pub async fn random(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
    recipe_type: RecipeType,
) -> imkitchen_shared::Result<Vec<(String, String)>> {
    let id = id.into();
    let statement = Query::select()
        .columns([MealPlanRecipe::Id, MealPlanRecipe::Name])
        .from(MealPlanRecipe::Table)
        .and_where(
            MealPlanRecipe::Id.into_column_ref().in_subquery(
                Query::select()
                    .columns([MealPlanRecipe::Id])
                    .from(MealPlanRecipe::Table)
                    .and_where(Expr::col(MealPlanRecipe::UserId).eq(id))
                    .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()))
                    .and_where(Expr::col(MealPlanRecipe::Name).is_not(""))
                    .order_by_expr(SimpleExpr::FunctionCall(Func::random()), Order::Asc)
                    .limit(7 * 4)
                    .take(),
            ),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let mut recipes = sqlx::query_as_with::<_, (String, String), _>(&sql, values)
        .fetch_all(pool)
        .await?;

    let mut rng = rand::rng();
    recipes.shuffle(&mut rng);

    Ok(recipes)
}

pub fn subscribe_command<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("mealplan-command")
        .handler(handle_generation_requested())
        .handler(handle_week_generated())
        .skip::<MealPlan, GenerationFailed>()
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
    let user_id = event.metadata.trigger_by().unwrap_or_default();

    if !has(&pool, &user_id, RecipeType::MainCourse).await? {
        evento::save::<MealPlan>(&user_id)
            .data(&GenerationFailed {
                reason: "No main course found".to_owned(),
                status: Status::Failed,
            })?
            .metadata(&event.metadata)?
            .commit(context.executor)
            .await?;

        return Ok(());
    }

    let main_course_recipes = random(&pool, &user_id, RecipeType::MainCourse).await?;
    let mut main_course_recipes = main_course_recipes.iter();

    let mut builder = evento::save::<MealPlan>(&user_id).metadata(&event.metadata)?;

    for (start, end) in event.data.weeks {
        let mut slots = vec![];

        while let Some(recipe) = main_course_recipes.by_ref().next() {
            if slots.len() == 7 {
                break;
            }

            let day = OffsetDateTime::from_unix_timestamp(start as i64)?
                + Duration::days((slots.len()) as i64);

            let appetizer_recipes = random(&pool, &user_id, RecipeType::Appetizer).await?;
            let mut appetizer_recipes = appetizer_recipes.iter();
            let dessert_recipes = random(&pool, &user_id, RecipeType::Dessert).await?;
            let mut dessert_recipes = dessert_recipes.iter();
            slots.push(Slot {
                day: day.unix_timestamp() as u64,
                appetizer: appetizer_recipes.next().map(|r| r.into()),
                main_course: recipe.into(),
                dessert: dessert_recipes.next().map(|r| r.into()),
                accompaniment: None,
            });
        }

        if slots.is_empty() {
            break;
        }

        builder = builder.data(&WeekGenerated {
            slots,
            start,
            end,
            status: Status::Idle,
        })?;
    }

    builder.commit(context.executor).await?;

    Ok(())
}

#[evento::handler(MealPlan)]
async fn handle_week_generated<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<WeekGenerated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let statement = Query::insert()
        .into_table(MealPlanLastWeek::Table)
        .columns([MealPlanLastWeek::UserId, MealPlanLastWeek::Start])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.data.start.to_owned().into(),
        ])
        .on_conflict(
            OnConflict::columns([MealPlanLastWeek::UserId])
                .update_column(MealPlanLastWeek::Start)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

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
