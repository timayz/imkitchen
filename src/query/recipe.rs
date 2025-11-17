use std::str::FromStr;

use bincode::{Decode, Encode};
use evento::{
    AggregatorName, Executor, SubscribeBuilder,
    cursor::{Args, ReadResult},
    sql::Reader,
};
use imkitchen_db::table::RecipePjt;
use imkitchen_recipe::{
    AccompanimentType, AdvancePrepChanged, BasicInformationChanged, Created, CuisineType,
    CuisineTypeChanged, Deleted, DietaryRestriction, DietaryRestrictionsChanged, Ingredient,
    IngredientsChanged, Instruction, InstructionsChanged, MadePrivate, MainCourseOptionsChanged,
    Recipe as RecipeAggregator, RecipeType, RecipeTypeChanged, SharedToCommunity,
};
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::prelude::FromRow;
use strum::{AsRefStr, EnumString};

#[derive(Debug, Encode, Decode)]
pub struct RecipeCursor {
    pub i: String,
    pub v: u64,
}

#[derive(Default)]
pub struct RecipeDetail {
    pub id: String,
    pub user_id: String,
    pub recipe_type: RecipeType,
    pub cuisine_type: CuisineType,
    pub name: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub accepts_accompaniment: bool,
    pub preferred_accompaniment_types: Vec<AccompanimentType>,
    pub advance_prep: String,
    pub is_shared: bool,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

impl<R: sqlx::Row> sqlx::FromRow<'_, R> for RecipeDetail
where
    i32: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    Vec<u8>: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    String: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    serde_json::Value: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    i64: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    u16: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    bool: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    for<'r> &'r str: sqlx::Type<R::Database> + sqlx::Decode<'r, R::Database>,
    for<'r> &'r str: sqlx::ColumnIndex<R>,
{
    fn from_row(row: &R) -> Result<Self, sqlx::Error> {
        let recipe_type: String = row.try_get("recipe_type")?;
        let cuisine_type: String = row.try_get("cuisine_type")?;
        let ingredients: Vec<u8> = row.try_get("ingredients")?;
        let instructions: Vec<u8> = row.try_get("instructions")?;
        let dietary_restrictions_json: serde_json::Value = row.try_get("dietary_restrictions")?;
        let preferred_accompaniment_types_json: serde_json::Value =
            row.try_get("preferred_accompaniment_types")?;

        let config = bincode::config::standard();
        let (ingredients, _) = bincode::decode_from_slice(&ingredients[..], config)
            .map_err(|e| sqlx::Error::InvalidArgument(e.to_string()))?;

        let (instructions, _) = bincode::decode_from_slice(&instructions[..], config)
            .map_err(|e| sqlx::Error::InvalidArgument(e.to_string()))?;

        let dietary_restrictions_vec = dietary_restrictions_json
            .as_array()
            .cloned()
            .unwrap_or_else(Vec::new);

        let mut dietary_restrictions = vec![];
        for restriction in dietary_restrictions_vec {
            dietary_restrictions.push(
                DietaryRestriction::from_str(restriction.as_str().unwrap_or_default())
                    .map_err(|err| sqlx::Error::InvalidArgument(err.to_string()))?,
            );
        }

        let preferred_accompaniment_types_vec = preferred_accompaniment_types_json
            .as_array()
            .cloned()
            .unwrap_or_else(Vec::new);

        let mut preferred_accompaniment_types = vec![];
        for preferred in preferred_accompaniment_types_vec {
            preferred_accompaniment_types.push(
                AccompanimentType::from_str(preferred.as_str().unwrap_or_default())
                    .map_err(|err| sqlx::Error::InvalidArgument(err.to_string()))?,
            );
        }
        let created_at: i64 = row.try_get("created_at")?;
        let updated_at: Option<i64> = row.try_get("updated_at")?;

        Ok(RecipeDetail {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            recipe_type: RecipeType::from_str(recipe_type.as_str())
                .map_err(|e| sqlx::Error::InvalidArgument(e.to_string()))?,
            cuisine_type: CuisineType::from_str(cuisine_type.as_str())
                .map_err(|e| sqlx::Error::InvalidArgument(e.to_string()))?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            prep_time: row.try_get("prep_time")?,
            cook_time: row.try_get("cook_time")?,
            ingredients,
            instructions,
            dietary_restrictions,
            accepts_accompaniment: row.try_get("accepts_accompaniment")?,
            preferred_accompaniment_types,
            advance_prep: row.try_get("advance_prep")?,
            is_shared: row.try_get("is_shared")?,
            created_at: created_at as u64,
            updated_at: updated_at.map(|v| v as u64),
        })
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub recipe_type: RecipeType,
    pub cuisine_type: CuisineType,
    pub name: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub accepts_accompaniment: bool,
    pub is_shared: bool,
    pub created_at: u64,
}

impl<R: sqlx::Row> sqlx::FromRow<'_, R> for Recipe
where
    String: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    serde_json::Value: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    i64: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    u16: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    bool: sqlx::Type<R::Database> + for<'r> sqlx::Decode<'r, R::Database>,
    for<'r> &'r str: sqlx::Type<R::Database> + sqlx::Decode<'r, R::Database>,
    for<'r> &'r str: sqlx::ColumnIndex<R>,
{
    fn from_row(row: &R) -> Result<Self, sqlx::Error> {
        let recipe_type: String = row.try_get("recipe_type")?;
        let cuisine_type: String = row.try_get("cuisine_type")?;
        let dietary_restrictions_json: serde_json::Value = row.try_get("dietary_restrictions")?;

        let dietary_restrictions_vec = dietary_restrictions_json
            .as_array()
            .cloned()
            .unwrap_or_else(Vec::new);

        let mut dietary_restrictions = vec![];
        for restriction in dietary_restrictions_vec {
            dietary_restrictions.push(
                DietaryRestriction::from_str(restriction.as_str().unwrap_or_default())
                    .map_err(|err| sqlx::Error::InvalidArgument(err.to_string()))?,
            );
        }

        let created_at: i64 = row.try_get("created_at")?;

        Ok(Recipe {
            id: row.try_get("id")?,
            recipe_type: RecipeType::from_str(recipe_type.as_str())
                .map_err(|e| sqlx::Error::InvalidArgument(e.to_string()))?,
            cuisine_type: CuisineType::from_str(cuisine_type.as_str())
                .map_err(|e| sqlx::Error::InvalidArgument(e.to_string()))?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            prep_time: row.try_get("prep_time")?,
            cook_time: row.try_get("cook_time")?,
            dietary_restrictions,
            accepts_accompaniment: row.try_get("accepts_accompaniment")?,
            is_shared: row.try_get("is_shared")?,
            created_at: created_at as u64,
        })
    }
}

impl evento::cursor::Cursor for Recipe {
    type T = RecipeCursor;

    fn serialize(&self) -> Self::T {
        Self::T {
            i: self.id.to_owned(),
            v: self.created_at,
        }
    }
}

impl evento::sql::Bind for Recipe {
    type T = RecipePjt;
    type I = [Self::T; 2];
    type V = [Expr; 2];
    type Cursor = Self;

    fn columns() -> Self::I {
        [RecipePjt::CreatedAt, RecipePjt::Id]
    }

    fn values(
        cursor: <<Self as evento::sql::Bind>::Cursor as evento::cursor::Cursor>::T,
    ) -> Self::V {
        [cursor.v.into(), cursor.i.into()]
    }
}

#[derive(Default, Debug, Deserialize, EnumString, AsRefStr)]
pub enum RecipeSortBy {
    #[default]
    RecentlyAdded,
}

pub struct RecipeInput {
    pub user_id: Option<String>,
    pub recipe_type: Option<RecipeType>,
    pub cuisine_type: Option<CuisineType>,
    pub is_shared: Option<bool>,
    pub sort_by: RecipeSortBy,
    pub args: Args,
}

pub async fn query_recipes(
    pool: &sqlx::SqlitePool,
    input: RecipeInput,
) -> anyhow::Result<ReadResult<Recipe>> {
    let mut statment = Query::select()
        .columns([
            RecipePjt::Id,
            RecipePjt::RecipeType,
            RecipePjt::CuisineType,
            RecipePjt::Name,
            RecipePjt::Description,
            RecipePjt::PrepTime,
            RecipePjt::CookTime,
            RecipePjt::DietaryRestrictions,
            RecipePjt::AcceptsAccompaniment,
            RecipePjt::IsShared,
            RecipePjt::CreatedAt,
        ])
        .from(RecipePjt::Table)
        .to_owned();

    if let Some(user_id) = input.user_id {
        statment.and_where(Expr::col(RecipePjt::UserId).eq(user_id));
    }

    if let Some(recipe_type) = input.recipe_type {
        statment.and_where(Expr::col(RecipePjt::RecipeType).eq(recipe_type.to_string()));
    }

    if let Some(cuisine_type) = input.cuisine_type {
        statment.and_where(Expr::col(RecipePjt::CuisineType).eq(cuisine_type.to_string()));
    }

    if let Some(is_shared) = input.is_shared {
        statment.and_where(Expr::col(RecipePjt::IsShared).eq(is_shared));
    }

    let mut reader = Reader::new(statment);

    if matches!(input.sort_by, RecipeSortBy::RecentlyAdded) {
        reader.desc();
    }

    Ok(reader
        .args(input.args)
        .execute::<_, Recipe, _>(pool)
        .await?)
}

pub async fn query_recipe_by_id(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
) -> anyhow::Result<Recipe> {
    let statment = Query::select()
        .columns([
            RecipePjt::Id,
            RecipePjt::RecipeType,
            RecipePjt::CuisineType,
            RecipePjt::Name,
            RecipePjt::Description,
            RecipePjt::PrepTime,
            RecipePjt::CookTime,
            RecipePjt::DietaryRestrictions,
            RecipePjt::AcceptsAccompaniment,
            RecipePjt::IsShared,
            RecipePjt::CreatedAt,
        ])
        .from(RecipePjt::Table)
        .and_where(Expr::col(RecipePjt::Id).eq(id.into()))
        .limit(1)
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with::<_, Recipe, _>(&sql, values)
        .fetch_one(pool)
        .await?)
}

pub async fn query_recipe_detail_by_id(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
) -> anyhow::Result<Option<RecipeDetail>> {
    let statment = Query::select()
        .columns([
            RecipePjt::Id,
            RecipePjt::UserId,
            RecipePjt::RecipeType,
            RecipePjt::CuisineType,
            RecipePjt::Name,
            RecipePjt::Description,
            RecipePjt::PrepTime,
            RecipePjt::CookTime,
            RecipePjt::Ingredients,
            RecipePjt::Instructions,
            RecipePjt::DietaryRestrictions,
            RecipePjt::AcceptsAccompaniment,
            RecipePjt::PreferredAccompanimentTypes,
            RecipePjt::AdvancePrep,
            RecipePjt::IsShared,
            RecipePjt::CreatedAt,
            RecipePjt::UpdatedAt,
        ])
        .from(RecipePjt::Table)
        .and_where(Expr::col(RecipePjt::Id).eq(id.into()))
        .limit(1)
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with::<_, RecipeDetail, _>(&sql, values)
        .fetch_optional(pool)
        .await?)
}

pub fn subscribe_recipe<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("recipe-query")
        .handler(handle_created())
        .handler(handle_recipe_type_changed())
        .handler(handle_basic_information_changed())
        .handler(handle_ingredients_changed())
        .handler(handle_instructions_changed())
        .handler(handle_dietary_restrictions_changed())
        .handler(handle_cuisine_type_changed())
        .handler(handle_main_course_options_changed())
        .handler(handle_advance_prep_changed())
        .handler(handle_shared_to_community())
        .handler(handle_made_private())
        .handler(handle_deleted())
        .handler_check_off()
}

#[evento::handler(RecipeAggregator)]
async fn handle_created<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let timestamp = event.timestamp;
    let aggregator_id = event.aggregator_id.clone();
    let user_id = event.metadata.trigger_by().unwrap_or_default();
    let name = event.data.name;
    let config = bincode::config::standard();
    let instructions = bincode::encode_to_vec(Vec::<Instruction>::default(), config)?;
    let ingredients = bincode::encode_to_vec(Vec::<Ingredient>::default(), config)?;

    let statment = Query::insert()
        .into_table(RecipePjt::Table)
        .columns([
            RecipePjt::Id,
            RecipePjt::UserId,
            RecipePjt::RecipeType,
            RecipePjt::CuisineType,
            RecipePjt::Name,
            RecipePjt::Ingredients,
            RecipePjt::Instructions,
            RecipePjt::DietaryRestrictions,
            RecipePjt::PreferredAccompanimentTypes,
            RecipePjt::CreatedAt,
        ])
        .values_panic([
            aggregator_id.into(),
            user_id.into(),
            RecipeType::default().to_string().into(),
            CuisineType::default().to_string().into(),
            name.into(),
            ingredients.into(),
            instructions.into(),
            serde_json::Value::Array(vec![]).into(),
            serde_json::Value::Array(vec![]).into(),
            timestamp.into(),
        ])
        .to_owned();
    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_recipe_type_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<RecipeTypeChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (
                RecipePjt::RecipeType,
                event.data.recipe_type.to_string().into(),
            ),
            (RecipePjt::UpdatedAt, event.timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_basic_information_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<BasicInformationChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let timestamp = event.timestamp;
    let aggregator_id = event.aggregator_id.clone();
    let name = event.data.name;
    let description = event.data.description;
    let prep_time = event.data.prep_time;
    let cook_time = event.data.cook_time;

    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::Name, name.into()),
            (RecipePjt::Description, description.into()),
            (RecipePjt::PrepTime, prep_time.into()),
            (RecipePjt::CookTime, cook_time.into()),
            (RecipePjt::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_ingredients_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<IngredientsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let ingredients = bincode::encode_to_vec(&event.data.ingredients, config)?;
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::Ingredients, ingredients.into()),
            (RecipePjt::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_instructions_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<InstructionsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let instructions = bincode::encode_to_vec(&event.data.instructions, config)?;
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::Instructions, instructions.into()),
            (RecipePjt::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_dietary_restrictions_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<DietaryRestrictionsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let dietary_restrictions = event
        .data
        .dietary_restrictions
        .iter()
        .map(|d| serde_json::Value::String(d.to_string()))
        .collect::<Vec<_>>();
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (
                RecipePjt::DietaryRestrictions,
                serde_json::Value::Array(dietary_restrictions).into(),
            ),
            (RecipePjt::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_cuisine_type_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<CuisineTypeChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (
                RecipePjt::CuisineType,
                event.data.cuisine_type.to_string().into(),
            ),
            (RecipePjt::UpdatedAt, event.timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_main_course_options_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MainCourseOptionsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let preferred_accompaniment_types = event
        .data
        .preferred_accompaniment_types
        .iter()
        .map(|a| serde_json::Value::String(a.to_string()))
        .collect::<Vec<_>>();
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (
                RecipePjt::AcceptsAccompaniment,
                event.data.accepts_accompaniment.into(),
            ),
            (
                RecipePjt::PreferredAccompanimentTypes,
                serde_json::Value::Array(preferred_accompaniment_types).into(),
            ),
            (RecipePjt::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_advance_prep_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<AdvancePrepChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let timestamp = event.timestamp;
    let aggregator_id = event.aggregator_id.clone();
    let description = event.data.description;

    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::AdvancePrep, description.into()),
            (RecipePjt::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_shared_to_community<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<SharedToCommunity>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::IsShared, event.data.shared.into()),
            (RecipePjt::UpdatedAt, event.timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_made_private<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MadePrivate>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::IsShared, event.data.shared.into()),
            (RecipePjt::UpdatedAt, event.timestamp.into()),
        ])
        .and_where(Expr::col(RecipePjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeAggregator)]
async fn handle_deleted<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::delete()
        .from_table(RecipePjt::Table)
        .and_where(Expr::col(RecipePjt::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
