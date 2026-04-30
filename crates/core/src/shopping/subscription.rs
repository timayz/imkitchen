use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::shopping_recipe::ShoppingRecipe;
use imkitchen_db::shopping_slot::ShoppingSlot;
use imkitchen_types::recipe::Ingredient;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("shopping")
        .handler(handle_recipe_created())
        .handler(handle_recipe_imported())
        .handler(handle_recipe_deleted())
        .handler(handle_mealplan_days_generated())
        .handler(handle_recipe_ingredients_changed())
}

#[evento::subscription]
async fn handle_mealplan_days_generated<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::mealplan::DaysGenerated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let mut statement = Query::insert()
        .into_table(ShoppingSlot::Table)
        .columns([
            ShoppingSlot::UserId,
            ShoppingSlot::Date,
            ShoppingSlot::RecipeIds,
        ])
        .to_owned();

    for slot in event.data.slots.iter() {
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

        let ids = bitcode::encode(&ids);

        statement.values_panic([
            event.metadata.requested_by()?.into(),
            slot.date.into(),
            ids.into(),
        ]);
    }

    statement.on_conflict(
        OnConflict::columns([ShoppingSlot::UserId, ShoppingSlot::Date])
            .update_column(ShoppingSlot::RecipeIds)
            .to_owned(),
    );

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

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
        ])
        .values([
            event.aggregator_id.to_owned().into(),
            event.metadata.requested_by()?.into(),
            ingredients.into(),
        ])?
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

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
        ])
        .values([
            event.aggregator_id.to_owned().into(),
            event.metadata.requested_by()?.into(),
            ingredients.into(),
        ])?
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

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
        .and_where(Expr::col(ShoppingRecipe::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

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
        &event.aggregator_id,
        ShoppingRecipe::Ingredients,
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
