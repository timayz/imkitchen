use evento::{Executor, ProjectionAggregate};
use imkitchen_db::shopping_slot::ShoppingSlot;
use imkitchen_types::shopping::{Generated, RecipeSetGenerated};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use std::collections::HashSet;
use validator::Validate;

use super::merge::merge_ingredients;

#[derive(Validate)]
pub struct Generate {
    pub date: u64,
    #[validate(range(min = 1, max = 30))]
    pub days: u8,
    pub household_size: u16,
}

impl<E: Executor> super::Module<E> {
    pub async fn generate(
        &self,
        input: Generate,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        input.validate()?;
        let request_by = request_by.into();
        let shopping = self
            .load(&request_by)
            .await?
            .unwrap_or_else(|| super::Shopping {
                user_id: request_by.to_owned(),
                checked: Default::default(),
                ingredients: Default::default(),
                recipes: Default::default(),
                cursor: Default::default(),
                from_date: 0,
                days: 0,
                generated_at: 0,
            });

        let slots_recipe_ids = self
            .filter_slot_recipe_ids(input.date, &request_by, input.days.into())
            .await?;

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(slots_recipe_ids.clone())
            .await?;

        let ingredients = merge_ingredients(recipe_ingredients, input.household_size);

        shopping
            .write()?
            .event(&Generated {
                ingredients,
                from_date: input.date,
                days: input.days,
            })
            .event(&RecipeSetGenerated {
                recipe_ids: slots_recipe_ids,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    async fn filter_slot_recipe_ids(
        &self,
        date: u64,
        user_id: impl Into<String>,
        limit: u64,
    ) -> anyhow::Result<Vec<String>> {
        let user_id = user_id.into();
        let statement = sea_query::Query::select()
            .column(ShoppingSlot::RecipeIds)
            .from(ShoppingSlot::Table)
            .and_where(Expr::col(ShoppingSlot::UserId).eq(&user_id))
            .and_where(Expr::col(ShoppingSlot::Date).gte(date))
            .limit(limit)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, (evento::sql_types::Bitcode<Vec<String>>,), _>(
                sqlx::AssertSqlSafe(sql),
                values,
            )
            .fetch_all(&self.read_db)
            .await?
            .into_iter()
            .flat_map(|ids| ids.0.0)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect(),
        )
    }
}
