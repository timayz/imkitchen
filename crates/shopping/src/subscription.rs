use std::collections::HashMap;

use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::ShoppingRecipe;
use imkitchen_shared::{recipe::Ingredient, shopping::Generated};
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("shopping")
        .handler(handle_recipe_created())
        .handler(handle_recipe_imported())
        .handler(handle_recipe_deleted())
        .handler(handle_recipe_basic_information_changed())
        .handler(handle_mealplan_week_generated())
        .handler(handle_recipe_ingredients_changed())
}

#[evento::sub_handler]
async fn handle_mealplan_week_generated<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::mealplan::WeekGenerated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let recipe_ids = event
        .data
        .slots
        .iter()
        .flat_map(|slot| {
            let mut ids = vec![slot.main_course.id.to_owned()];

            if let Some(ref r) = slot.appetizer {
                ids.push(r.id.to_owned());
            }

            if let Some(ref r) = slot.dessert {
                ids.push(r.id.to_owned());
            }

            if let Some(ref r) = slot.accompaniment {
                ids.push(r.id.to_owned());
            }

            ids
        })
        .collect::<Vec<_>>();

    let statement = Query::select()
        .columns([ShoppingRecipe::Ingredients, ShoppingRecipe::HouseholdSize])
        .from(ShoppingRecipe::Table)
        .and_where(Expr::col(ShoppingRecipe::UserId).eq(&event.aggregator_id))
        .and_where(Expr::col(ShoppingRecipe::Id).is_in(recipe_ids))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    let recipe_ingredients =
        sqlx::query_as_with::<_, (evento::sql_types::Bitcode<Vec<Ingredient>>, u16), _>(
            &sql, values,
        )
        .fetch_all(&pool)
        .await?;

    let mut ingredients = HashMap::new();
    for (recipe_ingredients, household_size) in recipe_ingredients {
        for ingredient in recipe_ingredients.0 {
            let entry = ingredients.entry(ingredient.key()).or_insert(Ingredient {
                name: ingredient.name,
                quantity: 0,
                unit: ingredient.unit,
                category: ingredient.category,
            });

            entry.quantity += ((event.data.household_size as u32 * ingredient.quantity
                / household_size as u32) as f64)
                .ceil() as u32;
        }
    }

    evento::aggregator(&event.aggregator_id)
        .original_version(event.version)
        .routing_key_opt(event.routing_key.to_owned())
        .event(&Generated {
            week: event.data.start,
            ingredients: ingredients.values().cloned().collect(),
        })
        .metadata(&event.metadata)
        .commit(context.executor)
        .await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let ingredients = bitcode::encode::<Vec<Ingredient>>(&vec![]);

    let statement = Query::insert()
        .into_table(ShoppingRecipe::Table)
        .columns([
            ShoppingRecipe::Id,
            ShoppingRecipe::UserId,
            ShoppingRecipe::Ingredients,
        ])
        .values([
            event.aggregator_id.to_owned().into(),
            event.metadata.user()?.into(),
            ingredients.into(),
        ])?
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
    let ingredients = bitcode::encode(&event.data.ingredients);

    let statement = Query::insert()
        .into_table(ShoppingRecipe::Table)
        .columns([
            ShoppingRecipe::Id,
            ShoppingRecipe::UserId,
            ShoppingRecipe::Ingredients,
        ])
        .values([
            event.aggregator_id.to_owned().into(),
            event.metadata.user()?.into(),
            ingredients.into(),
        ])?
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_deleted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::delete()
        .from_table(ShoppingRecipe::Table)
        .and_where(Expr::col(ShoppingRecipe::Id).eq(&event.aggregator_id))
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
    update_col(
        &pool,
        &event.aggregator_id,
        ShoppingRecipe::HouseholdSize,
        event.data.household_size,
    )
    .await?;

    Ok(())
}

#[evento::sub_handler]
async fn handle_recipe_ingredients_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_shared::recipe::IngredientsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let ingredients = bitcode::encode(&event.data.ingredients);

    update_col(
        &pool,
        &event.aggregator_id,
        ShoppingRecipe::HouseholdSize,
        ingredients,
    )
    .await?;

    Ok(())
}

async fn update_col(
    pool: &SqlitePool,
    id: impl Into<String>,
    col: ShoppingRecipe,
    value: impl Into<Expr>,
) -> anyhow::Result<()> {
    let statement = Query::update()
        .table(ShoppingRecipe::Table)
        .value(col, value)
        .and_where(Expr::col(ShoppingRecipe::Id).eq(id.into()))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}
