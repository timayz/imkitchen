use crate::{Generated, Shopping};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::ShoppingList;
use imkitchen_recipe::Ingredient;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

#[derive(Default, FromRow)]
pub struct ListWeekRow {
    pub week: u64,
    pub ingredients: imkitchen_db::types::Bincode<Vec<Ingredient>>,
}

impl super::Query {
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
            .fetch_optional(&self.0)
            .await?)
    }
}

pub fn subscribe_list<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("shopping-list")
        .handler(handle_generated())
        .handler_check_off()
}

#[evento::handler(Shopping)]
async fn handle_generated<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Generated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let ingredients = bincode::encode_to_vec(&event.data.ingredients, config)?;

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
