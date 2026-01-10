mod generate;

use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::MealPlanRecipe;
use imkitchen_shared::recipe::RecipeType;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;
use std::ops::Deref;

pub use generate::*;

pub struct Command<E: Executor> {
    state: imkitchen_shared::State<E>,
}

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<E: Executor> Command<E> {
    pub fn new(state: imkitchen_shared::State<E>) -> Self {
        Self { state }
    }
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("mealplan-command")
        .handler(handle_recipe_created())
        .handler(handle_recipe_imported())
        .handler(handle_recipe_deleted())
        .handler(handle_recipe_type_changed())
        .handler(handle_recipe_basic_information_changed())
        .handler(handle_recipe_dietary_restrictions_changed())
        .handler(handle_recipe_main_course_changed())
        .handler(handle_recipe_advance_prep_changed())
}

#[evento::sub_handler]
async fn handle_recipe_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let statement = Query::insert()
        .into_table(MealPlanRecipe::Table)
        .columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::UserId,
            MealPlanRecipe::RecipeType,
            MealPlanRecipe::Name,
            MealPlanRecipe::DietaryRestrictions,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.metadata.user()?.into(),
            RecipeType::default().to_string().into(),
            event.data.name.into(),
            serde_json::Value::Array(vec![]).into(),
        ])
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_imported<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::Imported>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let statement = Query::insert()
        .into_table(MealPlanRecipe::Table)
        .columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::UserId,
            MealPlanRecipe::RecipeType,
            MealPlanRecipe::Name,
            MealPlanRecipe::DietaryRestrictions,
            MealPlanRecipe::AdvancePrep,
            MealPlanRecipe::CookTime,
            MealPlanRecipe::PrepTime,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.metadata.user()?.into(),
            event.data.recipe_type.to_string().into(),
            event.data.name.into(),
            serde_json::Value::Array(vec![]).into(),
            event.data.advance_prep.into(),
            event.data.cook_time.into(),
            event.data.prep_time.into(),
        ])
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_type_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::RecipeTypeChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update_col(
        &pool,
        &event.aggregator_id,
        MealPlanRecipe::RecipeType,
        event.data.recipe_type.to_string(),
    )
    .await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_deleted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::delete()
        .from_table(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_basic_information_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::BasicInformationChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(MealPlanRecipe::Table)
        .value(MealPlanRecipe::Name, &event.data.name)
        .value(MealPlanRecipe::PrepTime, event.data.prep_time)
        .value(MealPlanRecipe::CookTime, event.data.cook_time)
        .and_where(Expr::col(MealPlanRecipe::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_dietary_restrictions_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::DietaryRestrictionsChanged>,
) -> anyhow::Result<()> {
    let dietary_restrictions = event
        .data
        .dietary_restrictions
        .iter()
        .map(|d| serde_json::Value::String(d.to_string()))
        .collect::<Vec<_>>();

    let pool = context.extract::<sqlx::SqlitePool>();
    update_col(
        &pool,
        &event.aggregator_id,
        MealPlanRecipe::DietaryRestrictions,
        serde_json::Value::Array(dietary_restrictions),
    )
    .await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_main_course_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::MainCourseOptionsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update_col(
        &pool,
        &event.aggregator_id,
        MealPlanRecipe::AcceptsAccompaniment,
        event.data.accepts_accompaniment,
    )
    .await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_advance_prep_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::AdvancePrepChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update_col(
        &pool,
        &event.aggregator_id,
        MealPlanRecipe::AdvancePrep,
        &event.data.advance_prep,
    )
    .await?;

    Ok(())
}

async fn update_col(
    pool: &SqlitePool,
    id: impl Into<String>,
    col: MealPlanRecipe,
    value: impl Into<Expr>,
) -> anyhow::Result<()> {
    let statement = Query::update()
        .table(MealPlanRecipe::Table)
        .value(col, value)
        .and_where(Expr::col(MealPlanRecipe::Id).eq(id.into()))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}
