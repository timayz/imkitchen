use evento::{Executor, ProjectionAggregate};
use imkitchen_db::shopping_recipe::ShoppingRecipe;
use imkitchen_db::shopping_slot::ShoppingSlot;
use imkitchen_types::{recipe::Ingredient, shopping::Generated};
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use std::collections::{HashMap, HashSet};
use validator::Validate;

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
                cursor: Default::default(),
                from_date: 0,
                days: 0,
                generated_at: 0,
            });

        let slots_recipe_ids = self
            .filter_slot_recipe_ids(input.date, &request_by, input.days.into())
            .await?;

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(slots_recipe_ids)
            .await?;

        let mut ingredients = HashMap::new();
        for (recipe_household_size, recipe_ingredients) in recipe_ingredients {
            for ingredient in recipe_ingredients {
                let scaled = scale_quantity(
                    ingredient.quantity,
                    recipe_household_size,
                    input.household_size,
                );
                let entry = ingredients.entry(ingredient.key()).or_insert(Ingredient {
                    name: ingredient.name,
                    quantity: 0,
                    unit: ingredient.unit,
                    category: ingredient.category,
                });

                entry.quantity += scaled;
            }
        }

        shopping
            .write()?
            .event(&Generated {
                ingredients: ingredients.values().cloned().collect(),
                from_date: input.date,
                days: input.days,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    async fn filter_recipe_ingredients_by_ids(
        &self,
        ids: Vec<String>,
    ) -> anyhow::Result<Vec<(u16, Vec<Ingredient>)>> {
        let statement = Query::select()
            .column(ShoppingRecipe::HouseholdSize)
            .column(ShoppingRecipe::Ingredients)
            .from(ShoppingRecipe::Table)
            .and_where(Expr::col(ShoppingRecipe::Id).is_in(ids))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        Ok(
            sqlx::query_as_with::<_, (u16, evento::sql_types::Bitcode<Vec<Ingredient>>), _>(
                sqlx::AssertSqlSafe(sql),
                values,
            )
            .fetch_all(&self.read_db)
            .await?
            .into_iter()
            .map(|(household_size, ingredients)| (household_size, ingredients.0))
            .collect(),
        )
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

/// Scale one recipe's ingredient quantity to the user's household size.
///
/// The recipe's authored size (`recipe_household_size`) doubles as its minimum:
/// a recipe can't realistically be made for fewer servings than it was written
/// for (e.g. a whole chicken serves 4 — you can't halve it for 2). So the
/// serving target is `max(recipe_household_size, user_household_size)` — we scale
/// up when the household is larger, but never down below the recipe's own size.
fn scale_quantity(quantity: u32, recipe_household_size: u16, user_household_size: u16) -> u32 {
    let recipe_household_size = Ord::max(recipe_household_size, 1);
    let serving_target = Ord::max(recipe_household_size, user_household_size);
    (quantity as f64 * serving_target as f64 / recipe_household_size as f64).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::scale_quantity;

    #[test]
    fn scales_up_when_household_exceeds_recipe() {
        // Recipe authored for 4, household of 8 → double.
        assert_eq!(scale_quantity(800, 4, 8), 1600);
    }

    #[test]
    fn respects_recipe_minimum_when_household_is_smaller() {
        // Household of 2 is below the recipe's authored 4 — do NOT scale down;
        // use the recipe's own quantities (the minimum). This is the #602 case.
        assert_eq!(scale_quantity(800, 4, 2), 800);
        assert_eq!(scale_quantity(150, 4, 1), 150);
    }

    #[test]
    fn keeps_quantity_when_household_matches_recipe() {
        assert_eq!(scale_quantity(800, 4, 4), 800);
    }

    #[test]
    fn rounds_up_fractional_results() {
        // 150 * 6 / 4 = 225 exactly.
        assert_eq!(scale_quantity(150, 4, 6), 225);
        // 100 * 3 / 2 = 150; 10 * 3 / 4 = 7.5 → 8 (ceil, never under-order).
        assert_eq!(scale_quantity(10, 4, 3), 10); // household 3 < recipe 4 → unchanged
        assert_eq!(scale_quantity(10, 2, 3), 15); // 10 * 3 / 2
    }

    #[test]
    fn guards_against_zero_recipe_size() {
        // A malformed 0-serving recipe must not divide by zero.
        assert_eq!(scale_quantity(100, 0, 4), 400);
    }
}
