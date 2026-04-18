use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::ShoppingList;
use imkitchen_shared::{recipe::Ingredient, shopping::Generated};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

#[derive(Default, FromRow)]
pub struct ShoppingListRow {
    pub ingredients: evento::sql_types::Bitcode<Vec<Ingredient>>,
    pub generated_at: u64,
}

impl<E: Executor> super::Query<E> {
    pub async fn find(
        &self,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<ShoppingListRow>> {
        let user_id = user_id.into();
        let statement = sea_query::Query::select()
            .columns([ShoppingList::Ingredients, ShoppingList::GeneratedAt])
            .from(ShoppingList::Table)
            .and_where(Expr::col(ShoppingList::UserId).eq(&user_id))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, ShoppingListRow, _>(&sql, values)
            .fetch_optional(&self.read_db)
            .await?)
    }
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("shopping-list").handler(handle_generated())
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
            ShoppingList::GeneratedAt,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            ingredients.into(),
            event.timestamp.into(),
        ])
        .on_conflict(
            OnConflict::column(ShoppingList::UserId)
                .update_columns([ShoppingList::Ingredients, ShoppingList::GeneratedAt])
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
