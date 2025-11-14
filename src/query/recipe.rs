use std::{fmt::Display, str::FromStr};

use bincode::{Decode, Encode};
use evento::{
    AggregatorName, Executor, SubscribeBuilder,
    cursor::{Args, ReadResult},
    sql::Reader,
};
use imkitchen_db::table::RecipePjt;
use imkitchen_recipe::{
    AdvancePreparationChanged, BasicInformationChanged, Created, CuisineType, CuisineTypeChanged,
    Deleted, DietaryRestrictionsChanged, IngredientsChanged, InstructionsChanged, MadePrivate,
    MainCourseOptionsChanged, Recipe as RecipeAggregator, RecipeType, RecipeTypeChanged,
    SharedToCommunity,
};
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::prelude::FromRow;

#[derive(Debug, Encode, Decode)]
pub struct RecipeCursor {
    pub i: String,
    pub v: i64,
}

#[derive(Debug, Default, Deserialize, FromRow)]
pub struct Recipe {
    pub id: String,
    pub user_id: String,
    pub recipe_type: String,
    pub cuisine_type: String,
    pub name: String,
    pub description: String,
    pub prep_time: i32,
    pub cook_time: i32,
    pub ingredients: String,
    pub instructions: String,
    pub dietary_restrictions: String,
    pub accept_accompaniments: bool,
    pub preferred_accompaniment_types: String,
    pub advance_preparation: String,
    pub is_shared: bool,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl Recipe {
    pub fn created_at(&self) -> String {
        super::format_relative_time(self.created_at as u64)
    }

    pub fn updated_at(&self) -> Option<String> {
        self.updated_at
            .map(|ts| super::format_relative_time(ts as u64))
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

#[derive(Debug, Deserialize)]
pub enum RecipeSortBy {
    MostRecent,
    OldestFirst,
}

impl Display for RecipeSortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for RecipeSortBy {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MostRecent" => Ok(Self::MostRecent),
            "OldestFirst" => Ok(Self::OldestFirst),
            _ => Err(()),
        }
    }
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
            RecipePjt::AcceptAccompaniments,
            RecipePjt::PreferredAccompanimentTypes,
            RecipePjt::AdvancePreparation,
            RecipePjt::IsShared,
            RecipePjt::CreatedAt,
            RecipePjt::UpdatedAt,
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

    if matches!(input.sort_by, RecipeSortBy::MostRecent) {
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
            RecipePjt::AcceptAccompaniments,
            RecipePjt::PreferredAccompanimentTypes,
            RecipePjt::AdvancePreparation,
            RecipePjt::IsShared,
            RecipePjt::CreatedAt,
            RecipePjt::UpdatedAt,
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
        .handler(handle_advance_preparation_changed())
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

    let statment = Query::insert()
        .into_table(RecipePjt::Table)
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
            RecipePjt::AcceptAccompaniments,
            RecipePjt::PreferredAccompanimentTypes,
            RecipePjt::AdvancePreparation,
            RecipePjt::IsShared,
            RecipePjt::CreatedAt,
        ])
        .values_panic([
            aggregator_id.into(),
            user_id.into(),
            RecipeType::default().to_string().into(),
            CuisineType::default().to_string().into(),
            name.into(),
            "".into(),
            0.into(),
            0.into(),
            "".into(),
            "".into(),
            "".into(),
            false.into(),
            "".into(),
            "".into(),
            false.into(),
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
            (RecipePjt::RecipeType, event.data.recipe_type.to_string().into()),
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
    let ingredients_str = event
        .data
        .ingredients
        .iter()
        .map(|i| format!("{}:{}:{}", i.name, i.unit, i.unit_type))
        .collect::<Vec<_>>()
        .join("|");
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::Ingredients, ingredients_str.into()),
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
    let instructions_str = event
        .data
        .instructions
        .iter()
        .map(|i| format!("{}:{}", i.description, i.time_before_next))
        .collect::<Vec<_>>()
        .join("|");
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::Instructions, instructions_str.into()),
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
    let dietary_restrictions_str = event
        .data
        .dietary_restrictions
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("|");
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::DietaryRestrictions, dietary_restrictions_str.into()),
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
            (RecipePjt::CuisineType, event.data.cuisine_type.to_string().into()),
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
    let preferred_accompaniment_types_str = event
        .data
        .preferred_accompaniment_types
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join("|");
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (
                RecipePjt::AcceptAccompaniments,
                event.data.accept_accompaniments.into(),
            ),
            (
                RecipePjt::PreferredAccompanimentTypes,
                preferred_accompaniment_types_str.into(),
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
async fn handle_advance_preparation_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<AdvancePreparationChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let timestamp = event.timestamp;
    let aggregator_id = event.aggregator_id.clone();
    let description = event.data.description;

    let statment = Query::update()
        .table(RecipePjt::Table)
        .values([
            (RecipePjt::AdvancePreparation, description.into()),
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
