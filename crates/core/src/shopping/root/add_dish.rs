use evento::{Executor, ProjectionAggregate};
use imkitchen_db::shopping_recipe::ShoppingRecipe;
use imkitchen_types::shopping::{DishAdded, DishPlaced, ServingsSet};
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

use super::{
    check_household_size,
    merge::{Servings, merge_ingredients},
};

impl<E: Executor> super::Module<E> {
    /// Add every recipe of one of the user's dishes to the list for
    /// `household_size` people, in the dish's meal order, recomputing the
    /// merged ingredients. Recipes already in the list stay where they are but
    /// are rescaled to the chosen size; deleted ones are skipped. The dish's
    /// recipes form one group in the kitchen (`DishPlaced`). Nothing is
    /// written when the list already holds the whole dish, grouped, at that
    /// size.
    pub async fn add_dish(
        &self,
        dish_id: impl Into<String>,
        household_size: u16,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let dish_id = dish_id.into();
        check_household_size(household_size)?;

        let dish = match crate::dish::load(&self.executor, &dish_id).await? {
            Some(dish) if dish.owner_id == request_by => dish,
            _ => crate::not_found!("dish"),
        };

        let shopping = self.load_or_empty(&request_by).await?;
        let existing = self.existing_recipe_ids(dish.recipe_ids).await?;

        let mut recipe_ids = shopping.recipes.clone();
        for id in &existing {
            if !recipe_ids.contains(id) {
                recipe_ids.push(id.clone());
            }
        }

        let grows = recipe_ids.len() != shopping.recipes.len();
        let rescales = existing
            .iter()
            .any(|id| shopping.servings.get(id) != Some(&household_size));
        let grouped = shopping
            .dish(&dish_id)
            .is_some_and(|d| d.recipe_ids == existing);
        if !grows && !rescales && grouped {
            return Ok(());
        }

        let mut builder = shopping.write()?.requested_by(request_by).to_owned();

        if grows {
            let mut per_recipe = shopping.servings.clone();
            for id in &existing {
                per_recipe.insert(id.clone(), household_size);
            }
            let recipe_ingredients = self
                .filter_recipe_ingredients_by_ids(recipe_ids.clone())
                .await?;
            let ingredients = merge_ingredients(
                recipe_ingredients,
                &Servings {
                    per_recipe: &per_recipe,
                    default: household_size,
                },
            );
            builder.event(&DishAdded {
                dish_id: dish_id.clone(),
                recipe_ids,
                ingredients,
            });
        }

        if !grouped && !existing.is_empty() {
            builder.event(&DishPlaced {
                dish_id,
                recipe_ids: existing.clone(),
            });
        }

        builder.event(&ServingsSet {
            recipe_ids: existing,
            household_size,
        });
        builder.commit(&self.executor).await?;

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
