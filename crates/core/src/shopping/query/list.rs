use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::shopping_list::ShoppingList;
use imkitchen_types::{
    recipe::Ingredient,
    shopping::{Generated, RecipeAdded, RecipeRemoved, RecipeSetGenerated},
};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

#[derive(Default, FromRow)]
pub struct ShoppingListRow {
    pub ingredients: evento::sql_types::Bitcode<Vec<Ingredient>>,
    pub from_date: u64,
    pub days: u8,
    pub generated_at: u64,
    // Nullable: pre-migration rows and freshly-generated lists may have no
    // recipe id blob yet. Absent → empty recipe set.
    pub recipes: Option<evento::sql_types::Bitcode<Vec<String>>>,
}

impl<E: Executor> crate::shopping::Module<E> {
    pub async fn find(
        &self,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<ShoppingListRow>> {
        let user_id = user_id.into();
        let statement = sea_query::Query::select()
            .columns([
                ShoppingList::Ingredients,
                ShoppingList::FromDate,
                ShoppingList::Days,
                ShoppingList::GeneratedAt,
                ShoppingList::Recipes,
            ])
            .from(ShoppingList::Table)
            .and_where(Expr::col(ShoppingList::UserId).eq(&user_id))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, ShoppingListRow, _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_optional(&self.read_db)
                .await?,
        )
    }
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("shopping-list")
        .handler(handle_generated())
        .handler(handle_recipe_set_generated())
        .handler(handle_recipe_added())
        .handler(handle_recipe_removed())
}

#[evento::subscription]
async fn handle_generated<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Generated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let ingredients = bitcode::encode(&event.data.ingredients);

    let statement = Query::insert()
        .into_table(ShoppingList::Table)
        .columns([
            ShoppingList::UserId,
            ShoppingList::Ingredients,
            ShoppingList::FromDate,
            ShoppingList::Days,
            ShoppingList::GeneratedAt,
        ])
        .values_panic([
            event.aggregate_id.to_owned().into(),
            ingredients.into(),
            event.data.from_date.into(),
            (event.data.days as i32).into(),
            event.timestamp.into(),
        ])
        .on_conflict(
            OnConflict::column(ShoppingList::UserId)
                .update_columns([
                    ShoppingList::Ingredients,
                    ShoppingList::FromDate,
                    ShoppingList::Days,
                    ShoppingList::GeneratedAt,
                ])
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Persist the recipe id set that fed a generated list. Committed alongside
/// `Generated`; upsert on `UserId` so it works regardless of arrival order.
#[evento::subscription]
async fn handle_recipe_set_generated<E: Executor>(
    context: &Context<'_, E>,
    event: Event<RecipeSetGenerated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let recipes = bitcode::encode(&event.data.recipe_ids);

    let statement = Query::insert()
        .into_table(ShoppingList::Table)
        .columns([
            ShoppingList::UserId,
            ShoppingList::Ingredients,
            ShoppingList::GeneratedAt,
            ShoppingList::Recipes,
        ])
        .values_panic([
            event.aggregate_id.to_owned().into(),
            bitcode::encode::<Vec<Ingredient>>(&vec![]).into(),
            event.timestamp.into(),
            recipes.into(),
        ])
        .on_conflict(
            OnConflict::column(ShoppingList::UserId)
                .update_column(ShoppingList::Recipes)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_recipe_added<E: Executor>(
    context: &Context<'_, E>,
    event: Event<RecipeAdded>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    upsert_recipe_change(
        &pool,
        &event.aggregate_id,
        &event.data.ingredients,
        &event.data.recipe_ids,
        event.timestamp,
    )
    .await
}

#[evento::subscription]
async fn handle_recipe_removed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<RecipeRemoved>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    upsert_recipe_change(
        &pool,
        &event.aggregate_id,
        &event.data.ingredients,
        &event.data.recipe_ids,
        event.timestamp,
    )
    .await
}

/// Shared upsert for manual add/remove: replaces the merged ingredient list and
/// the recipe id set. Leaves `FromDate`/`Days` (the meal-plan window) untouched.
async fn upsert_recipe_change(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    ingredients: &[Ingredient],
    recipe_ids: &[String],
    timestamp: u64,
) -> anyhow::Result<()> {
    let ingredients = bitcode::encode(&ingredients.to_vec());
    let recipes = bitcode::encode(&recipe_ids.to_vec());

    let statement = Query::insert()
        .into_table(ShoppingList::Table)
        .columns([
            ShoppingList::UserId,
            ShoppingList::Ingredients,
            ShoppingList::GeneratedAt,
            ShoppingList::Recipes,
        ])
        .values_panic([
            user_id.to_owned().into(),
            ingredients.clone().into(),
            timestamp.into(),
            recipes.clone().into(),
        ])
        .on_conflict(
            OnConflict::column(ShoppingList::UserId)
                .update_columns([
                    ShoppingList::Ingredients,
                    ShoppingList::GeneratedAt,
                    ShoppingList::Recipes,
                ])
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(pool)
        .await?;

    Ok(())
}
