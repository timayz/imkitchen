use crate::shopping_list::{Generated, ShoppingList};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::MealPlanShoppingList;
use imkitchen_shared::Event;
use sea_query::{OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

pub fn subscribe_shopping_list<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("mealplan-shopping-list")
        .handler(handle_generated())
        .handler_check_off()
}

#[evento::handler(ShoppingList)]
async fn handle_generated<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Generated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let ingredients = bincode::encode_to_vec(&event.data.ingredients, config)?;

    let statement = Query::insert()
        .into_table(MealPlanShoppingList::Table)
        .columns([
            MealPlanShoppingList::UserId,
            MealPlanShoppingList::Week,
            MealPlanShoppingList::Ingredients,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.data.week.into(),
            ingredients.into(),
        ])
        .on_conflict(
            OnConflict::columns([MealPlanShoppingList::UserId, MealPlanShoppingList::Week])
                .update_column(MealPlanShoppingList::Ingredients)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
