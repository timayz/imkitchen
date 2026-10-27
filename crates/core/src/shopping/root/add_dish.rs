use evento::{Executor, ProjectionAggregate};
use imkitchen_db::shopping_recipe::ShoppingRecipe;
use imkitchen_types::shopping::DishAdded;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

use super::merge::merge_ingredients;

impl<E: Executor> super::Module<E> {
    /// Add every recipe of one of the user's dishes to the list, in the
    /// dish's meal order, recomputing the merged ingredients. Recipes
    /// already in the list stay where they are; deleted ones are skipped.
    /// Nothing is written when the list already holds the whole dish.
    pub async fn add_dish(
        &self,
        dish_id: impl Into<String>,
        household_size: u16,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let dish_id = dish_id.into();

        let dish = match crate::dish::load(&self.executor, &dish_id).await? {
            Some(dish) if dish.owner_id == request_by => dish,
            _ => crate::not_found!("dish"),
        };

        let shopping = self.load_or_empty(&request_by).await?;
        let existing = self.existing_recipe_ids(dish.recipe_ids).await?;

        let mut recipe_ids = shopping.recipes.clone();
        for id in existing {
            if !recipe_ids.contains(&id) {
                recipe_ids.push(id);
            }
        }

        if recipe_ids.len() == shopping.recipes.len() {
            return Ok(());
        }

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(recipe_ids.clone())
            .await?;
        let ingredients = merge_ingredients(recipe_ingredients, household_size);

        shopping
            .write()?
            .event(&DishAdded {
                dish_id,
                recipe_ids,
                ingredients,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    /// `ids` that still exist in the `shopping_recipe` projection, in the
    /// given order.
    async fn existing_recipe_ids(&self, ids: Vec<String>) -> anyhow::Result<Vec<String>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let statement = Query::select()
            .column(ShoppingRecipe::Id)
            .from(ShoppingRecipe::Table)
            .and_where(Expr::col(ShoppingRecipe::Id).is_in(ids.clone()))
            .to_owned();
        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        let found: Vec<String> =
            sqlx::query_scalar_with::<_, String, _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_all(&self.read_db)
                .await?;

        Ok(ids.into_iter().filter(|id| found.contains(id)).collect())
    }
}
