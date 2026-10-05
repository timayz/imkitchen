//! Candidate pool for list generation: the `meal_plan_recipe` table holds the
//! user's own recipes plus the community recipes they saved, with the fields
//! the picker filters on (type, dietary restrictions, accompaniment flag).
//!
//! The subscription key is kept as `"mealplan-command"` from the calendar
//! era so the existing cursor and table carry over unchanged.

use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::mealplan_recipe::MealPlanRecipe;
use imkitchen_types::recipe::RecipeType;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

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
        .handler(handle_favorite_saved())
        .handler(handle_favorite_unsaved())
}

#[evento::subscription]
async fn handle_recipe_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::Created>,
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
            event.aggregate_id.to_owned().into(),
            event.metadata.requested_by()?.into(),
            RecipeType::default().to_string().into(),
            event.data.name.into(),
            serde_json::Value::Array(vec![]).into(),
        ])
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_imported<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::Imported>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let dietary_restrictions = event
        .data
        .dietary_restrictions
        .iter()
        .map(|d| serde_json::Value::String(d.to_string()))
        .collect::<Vec<_>>();

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
            MealPlanRecipe::AcceptsAccompaniment,
        ])
        .values_panic([
            event.aggregate_id.to_owned().into(),
            event.metadata.requested_by()?.into(),
            event.data.recipe_type.to_string().into(),
            event.data.name.into(),
            serde_json::Value::Array(dietary_restrictions).into(),
            event.data.advance_prep.into(),
            event.data.cook_time.into(),
            event.data.prep_time.into(),
            event.data.accepts_accompaniment.into(),
        ])
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_type_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::RecipeTypeChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update_col(
        &pool,
        &event.aggregate_id,
        MealPlanRecipe::RecipeType,
        event.data.recipe_type.to_string(),
    )
    .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_deleted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::delete()
        .from_table(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::Id).eq(&event.aggregate_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_basic_information_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::BasicInformationChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(MealPlanRecipe::Table)
        .value(MealPlanRecipe::Name, &event.data.name)
        .value(MealPlanRecipe::PrepTime, event.data.prep_time)
        .value(MealPlanRecipe::CookTime, event.data.cook_time)
        .and_where(Expr::col(MealPlanRecipe::Id).eq(&event.aggregate_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_dietary_restrictions_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::DietaryRestrictionsChanged>,
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
        &event.aggregate_id,
        MealPlanRecipe::DietaryRestrictions,
        serde_json::Value::Array(dietary_restrictions),
    )
    .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_main_course_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::MainCourseOptionsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update_col(
        &pool,
        &event.aggregate_id,
        MealPlanRecipe::AcceptsAccompaniment,
        event.data.accepts_accompaniment,
    )
    .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_advance_prep_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::AdvancePrepChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update_col(
        &pool,
        &event.aggregate_id,
        MealPlanRecipe::AdvancePrep,
        &event.data.advance_prep,
    )
    .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_favorite_saved<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::favorite::Saved>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let select = Query::select()
        .from(MealPlanRecipe::Table)
        .columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::RecipeType,
            MealPlanRecipe::Name,
            MealPlanRecipe::DietaryRestrictions,
            MealPlanRecipe::AdvancePrep,
            MealPlanRecipe::CookTime,
            MealPlanRecipe::PrepTime,
            MealPlanRecipe::AcceptsAccompaniment,
        ])
        .expr(Expr::value(event.metadata.requested_by()?))
        .and_where(Expr::col(MealPlanRecipe::Id).eq(&event.data.recipe_id))
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(&event.data.recipe_owner))
        .to_owned();

    let statement = Query::insert()
        .into_table(MealPlanRecipe::Table)
        .columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::RecipeType,
            MealPlanRecipe::Name,
            MealPlanRecipe::DietaryRestrictions,
            MealPlanRecipe::AdvancePrep,
            MealPlanRecipe::CookTime,
            MealPlanRecipe::PrepTime,
            MealPlanRecipe::AcceptsAccompaniment,
            MealPlanRecipe::UserId,
        ])
        .select_from(select)?
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_favorite_unsaved<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::favorite::Unsaved>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::delete()
        .from_table(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::Id).eq(&event.data.recipe_id))
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(&event.metadata.requested_by()?))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
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
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await?;

    Ok(())
}
