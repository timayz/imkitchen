use evento::Executor;
use imkitchen_db::shopping_recipe::ShoppingRecipe;
use imkitchen_types::recipe::Ingredient;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use std::collections::HashMap;

impl<E: Executor> super::Module<E> {
    /// Fetch each recipe's authored household size and ingredient list from the
    /// `shopping_recipe` projection, for a set of recipe ids.
    pub(crate) async fn filter_recipe_ingredients_by_ids(
        &self,
        ids: Vec<String>,
    ) -> anyhow::Result<Vec<(u16, Vec<Ingredient>)>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

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

    /// Whether a `shopping_recipe` row exists for the given recipe id. Ownership
    /// is intentionally NOT checked here: a user may add a shared recipe they do
    /// not own (viewability is enforced in the web layer, like `save()`).
    pub(crate) async fn recipe_exists(&self, recipe_id: &str) -> anyhow::Result<bool> {
        let statement = Query::select()
            .expr(Expr::val(1))
            .from(ShoppingRecipe::Table)
            .and_where(Expr::col(ShoppingRecipe::Id).eq(recipe_id))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        Ok(
            sqlx::query_scalar_with::<_, i32, _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_optional(&self.read_db)
                .await?
                .is_some(),
        )
    }
}

/// Merge and scale a set of recipes' ingredients into a single shopping list.
///
/// Duplicate ingredients (same `key()`) are summed. Each recipe's quantities are
/// scaled from its authored household size to the user's household size via
/// [`scale_quantity`].
pub(crate) fn merge_ingredients(
    recipe_ingredients: Vec<(u16, Vec<Ingredient>)>,
    user_household_size: u16,
) -> Vec<Ingredient> {
    let mut ingredients: HashMap<String, Ingredient> = HashMap::new();
    for (recipe_household_size, list) in recipe_ingredients {
        for ingredient in list {
            let scaled = scale_quantity(
                ingredient.quantity,
                recipe_household_size,
                user_household_size,
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

    ingredients.into_values().collect()
}

/// Scale one recipe's ingredient quantity to the user's household size.
///
/// The recipe's authored size (`recipe_household_size`) doubles as its minimum:
/// a recipe can't realistically be made for fewer servings than it was written
/// for (e.g. a whole chicken serves 4 — you can't halve it for 2). So the
/// serving target is `max(recipe_household_size, user_household_size)` — we scale
/// up when the household is larger, but never down below the recipe's own size.
pub(crate) fn scale_quantity(
    quantity: u32,
    recipe_household_size: u16,
    user_household_size: u16,
) -> u32 {
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
