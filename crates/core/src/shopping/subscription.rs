use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::shopping_recipe::ShoppingRecipe;
use imkitchen_types::recipe::Ingredient;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("shopping")
        .handler(handle_recipe_created())
        .handler(handle_recipe_imported())
        .handler(handle_recipe_deleted())
        .handler(handle_recipe_ingredients_changed())
        .handler(handle_recipe_basic_information_changed())
}

/// Recipes are authored for this many servings by default (matches the
/// `recipe_user` projection default).
const DEFAULT_HOUSEHOLD_SIZE: u16 = 4;

#[evento::subscription]
async fn handle_recipe_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let ingredients = bitcode::encode::<Vec<Ingredient>>(&vec![]);

    let statement = Query::insert()
        .into_table(ShoppingRecipe::Table)
        .columns([
            ShoppingRecipe::Id,
            ShoppingRecipe::UserId,
            ShoppingRecipe::Ingredients,
            ShoppingRecipe::HouseholdSize,
        ])
        .values([
            event.aggregate_id.to_owned().into(),
            event.metadata.requested_by()?.into(),
            ingredients.into(),
            DEFAULT_HOUSEHOLD_SIZE.into(),
        ])?
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
    let ingredients = bitcode::encode(&event.data.ingredients);

    let statement = Query::insert()
        .into_table(ShoppingRecipe::Table)
        .columns([
            ShoppingRecipe::Id,
            ShoppingRecipe::UserId,
            ShoppingRecipe::Ingredients,
            ShoppingRecipe::HouseholdSize,
        ])
        .values([
            event.aggregate_id.to_owned().into(),
            event.metadata.requested_by()?.into(),
            ingredients.into(),
            event.data.household_size.into(),
        ])?
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
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
        .from_table(ShoppingRecipe::Table)
        .and_where(Expr::col(ShoppingRecipe::Id).eq(&event.aggregate_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_ingredients_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::IngredientsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let ingredients = bitcode::encode(&event.data.ingredients);

    update_col(
        &pool,
        &event.aggregate_id,
        ShoppingRecipe::Ingredients,
        ingredients,
    )
    .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_basic_information_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::BasicInformationChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    update_col(
        &pool,
        &event.aggregate_id,
        ShoppingRecipe::HouseholdSize,
        event.data.household_size,
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
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await?;

    Ok(())
}
