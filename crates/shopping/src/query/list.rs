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
use time::OffsetDateTime;

#[derive(Default, FromRow)]
pub struct ListWeekRow {
    pub week: u64,
    pub ingredients: evento::sql_types::Bitcode<Vec<Ingredient>>,
}

impl<E: Executor> super::Query<E> {
    pub async fn next_from(
        &self,
        week: u64,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<ListWeekRow>> {
        let user_id = user_id.into();
        let week = OffsetDateTime::from_unix_timestamp(week.try_into()?)?;
        let statement = sea_query::Query::select()
            .columns([ShoppingList::Week, ShoppingList::Ingredients])
            .from(ShoppingList::Table)
            .and_where(Expr::col(ShoppingList::UserId).eq(&user_id))
            .and_where(Expr::col(ShoppingList::Week).gte(week.unix_timestamp()))
            .order_by_expr(Expr::col(ShoppingList::Week), sea_query::Order::Asc)
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, ListWeekRow, _>(&sql, values)
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
            ShoppingList::Week,
            ShoppingList::Ingredients,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.data.week.into(),
            ingredients.into(),
        ])
        .on_conflict(
            OnConflict::columns([ShoppingList::UserId, ShoppingList::Week])
                .update_column(ShoppingList::Ingredients)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
