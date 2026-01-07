use evento::{
    Executor, Projection,
    metadata::{Event, Metadata},
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::MealPlanRecipe;
use imkitchen_recipe::{DietaryRestriction, RecipeType};
use rand::seq::SliceRandom;
use sea_query::{Expr, ExprTrait, Func, IntoColumnRef, Query, SimpleExpr, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};
use time::{Duration, OffsetDateTime};

use crate::{Slot, SlotRecipe, WeekGenerated};

#[evento::command]
pub struct Command {}

#[derive(Clone, FromRow)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub accepts_accompaniment: bool,
}

impl From<&Recipe> for SlotRecipe {
    fn from(value: &Recipe) -> Self {
        SlotRecipe {
            id: value.id.to_owned(),
            name: value.name.to_owned(),
        }
    }
}

pub struct Randomize {
    pub cuisine_variety_weight: f32,
    pub dietary_restrictions: Vec<imkitchen_recipe::DietaryRestriction>,
}

pub struct Generate {
    pub user_id: String,
    pub weeks: Vec<(u64, u64)>,
    pub randomize: Option<Randomize>,
}

impl<'a, E: Executor> Command<'a, E> {
    pub async fn generate(
        executor: &E,
        pool: &SqlitePool,
        input: Generate,
    ) -> imkitchen_shared::Result<()> {
        let main_course_recipes = match input.randomize.as_ref() {
            Some(opts) => {
                random(
                    pool,
                    &input.user_id,
                    RecipeType::MainCourse,
                    opts.cuisine_variety_weight,
                    opts.dietary_restrictions.to_vec(),
                )
                .await?
            }
            _ => first_week_recipes(pool, &input.user_id, RecipeType::MainCourse).await?,
        };

        if main_course_recipes.is_empty() {
            imkitchen_shared::user!("No main course found");
        }

        let mut main_course_recipes = main_course_recipes.iter().cycle().take(7 * 4);
        let mut builder = evento::aggregator(&input.user_id)
            .metadata(&Metadata::new(&input.user_id))
            .to_owned();

        for (start, end) in input.weeks {
            let mut slots = vec![];

            while let Some(recipe) = main_course_recipes.by_ref().next() {
                let day = OffsetDateTime::from_unix_timestamp(start as i64)?
                    + Duration::days((slots.len()) as i64);

                let appetizer_recipes = match input.randomize.as_ref() {
                    Some(opts) => {
                        random(
                            pool,
                            &input.user_id,
                            RecipeType::Appetizer,
                            1.0,
                            opts.dietary_restrictions.to_vec(),
                        )
                        .await?
                    }
                    _ => first_week_recipes(pool, &input.user_id, RecipeType::Appetizer).await?,
                };

                let mut appetizer_recipes = appetizer_recipes.iter();

                let accompaniment_recipes = match input.randomize.as_ref() {
                    Some(opts) => {
                        random(
                            pool,
                            &input.user_id,
                            RecipeType::Accompaniment,
                            1.0,
                            opts.dietary_restrictions.to_vec(),
                        )
                        .await?
                    }
                    _ => {
                        first_week_recipes(pool, &input.user_id, RecipeType::Accompaniment).await?
                    }
                };

                let mut accompaniment_recipes = accompaniment_recipes.iter();

                let dessert_recipes = match input.randomize.as_ref() {
                    Some(opts) => {
                        random(
                            pool,
                            &input.user_id,
                            RecipeType::Dessert,
                            1.0,
                            opts.dietary_restrictions.to_vec(),
                        )
                        .await?
                    }
                    _ => first_week_recipes(pool, &input.user_id, RecipeType::Dessert).await?,
                };
                let mut dessert_recipes = dessert_recipes.iter();

                let accompaniment = if recipe.accepts_accompaniment {
                    accompaniment_recipes.next().map(|r| r.into())
                } else {
                    None
                };

                slots.push(Slot {
                    day: day.unix_timestamp() as u64,
                    appetizer: appetizer_recipes.next().map(|r| r.into()),
                    main_course: recipe.into(),
                    dessert: dessert_recipes.next().map(|r| r.into()),
                    accompaniment,
                });

                if slots.len() == 7 {
                    break;
                }
            }

            if slots.is_empty() {
                break;
            }

            builder.event(&WeekGenerated { slots, start, end });
        }

        builder.commit(executor).await?;

        Ok(())
    }
}

async fn random(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
    recipe_type: RecipeType,
    weight: f32,
    dietary_restrictions: Vec<DietaryRestriction>,
) -> imkitchen_shared::Result<Vec<Recipe>> {
    if weight < 0.1 {
        imkitchen_shared::user!("weight must be greater than or equal to 0.1");
    }

    let id = id.into();
    let mut sub_statement = Query::select()
        .columns([MealPlanRecipe::Id])
        .from(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(id))
        .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()))
        .and_where(Expr::col(MealPlanRecipe::Name).is_not(""))
        .to_owned();

    if !dietary_restrictions.is_empty() {
        let in_clause = dietary_restrictions
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");

        sub_statement.and_where(Expr::cust_with_values(
            format!(
                "(SELECT COUNT(*) FROM json_each(dietary_restrictions) WHERE value IN ({})) = ?",
                in_clause
            ),
            dietary_restrictions
                .iter()
                .map(|t| sea_query::Value::String(Some(*Box::new(t.to_string()))))
                .chain(std::iter::once(sea_query::Value::Int(Some(
                    dietary_restrictions.len() as i32,
                ))))
                .collect::<Vec<_>>(),
        ));
    }

    sub_statement
        .order_by_expr(
            SimpleExpr::FunctionCall(Func::random()),
            sea_query::Order::Asc,
        )
        .limit(7 * 4);

    let statement = Query::select()
        .columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::Name,
            MealPlanRecipe::AcceptsAccompaniment,
        ])
        .from(MealPlanRecipe::Table)
        .and_where(
            MealPlanRecipe::Id
                .into_column_ref()
                .in_subquery(sub_statement),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let mut recipes = sqlx::query_as_with::<_, Recipe, _>(&sql, values)
        .fetch_all(pool)
        .await?;

    let mut rng = rand::rng();
    recipes.shuffle(&mut rng);
    recipes.truncate((recipes.len() as f32 * weight).ceil() as usize);

    Ok(recipes)
}

async fn first_week_recipes(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
    recipe_type: RecipeType,
) -> imkitchen_shared::Result<Vec<Recipe>> {
    let id = id.into();

    let statement = Query::select()
        .columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::Name,
            MealPlanRecipe::AcceptsAccompaniment,
        ])
        .from(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(id))
        .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()))
        .and_where(Expr::col(MealPlanRecipe::Name).is_not(""))
        .limit(7)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let row = sqlx::query_as_with::<_, Recipe, _>(&sql, values)
        .fetch_all(pool)
        .await?;

    let mut recipes = vec![];

    for _ in 0..3 {
        let mut rng = rand::rng();
        let mut r = row.to_vec();
        r.shuffle(&mut rng);
        recipes.extend(r);
    }

    Ok(recipes)
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
}

#[evento::sub_handler]
async fn handle_recipe_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_recipe::Created>,
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
    event: Event<imkitchen_recipe::Imported>,
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
            event.data.recipe_type.to_string().into(),
            event.data.name.into(),
            serde_json::Value::Array(vec![]).into(),
        ])
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_type_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_recipe::RecipeTypeChanged>,
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
    event: Event<imkitchen_recipe::Deleted>,
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
    event: Event<imkitchen_recipe::BasicInformationChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update_col(
        &pool,
        &event.aggregator_id,
        MealPlanRecipe::Name,
        &event.data.name,
    )
    .await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_dietary_restrictions_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_recipe::DietaryRestrictionsChanged>,
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
    event: Event<imkitchen_recipe::MainCourseOptionsChanged>,
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
