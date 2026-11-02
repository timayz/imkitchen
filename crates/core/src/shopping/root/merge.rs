use evento::Executor;
use imkitchen_db::shopping_recipe::ShoppingRecipe;
use imkitchen_types::recipe::Ingredient;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use std::collections::HashMap;

impl<E: Executor> super::Module<E> {
    /// Fetch each recipe's id, authored household size and ingredient list
    /// from the `shopping_recipe` projection, for a set of recipe ids.
    pub(crate) async fn filter_recipe_ingredients_by_ids(
        &self,
        ids: Vec<String>,
    ) -> anyhow::Result<Vec<RecipeIngredients>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let statement = Query::select()
            .column(ShoppingRecipe::Id)
            .column(ShoppingRecipe::HouseholdSize)
            .column(ShoppingRecipe::Ingredients)
            .from(ShoppingRecipe::Table)
            .and_where(Expr::col(ShoppingRecipe::Id).is_in(ids))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        Ok(sqlx::query_as_with::<
            _,
            (String, u16, evento::sql_types::Bitcode<Vec<Ingredient>>),
            _,
        >(sqlx::AssertSqlSafe(sql), values)
        .fetch_all(&self.read_db)
        .await?
        .into_iter()
        .map(|(id, household_size, ingredients)| (id, household_size, ingredients.0))
        .collect())
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

/// `(recipe id, authored household size, ingredients)` as read from
/// `shopping_recipe`.
pub(crate) type RecipeIngredients = (String, u16, Vec<Ingredient>);

/// The household size each listed recipe is scaled to: the size recorded when
/// it was generated or added, or `default` (the user's settings value) for a
/// recipe that predates per-recipe servings.
pub struct Servings<'a> {
    pub per_recipe: &'a HashMap<String, u16>,
    pub default: u16,
}

impl Servings<'_> {
    pub fn of(&self, recipe_id: &str) -> u16 {
        self.per_recipe
            .get(recipe_id)
            .copied()
            .unwrap_or(self.default)
    }
}

/// Merge and scale a set of recipes' ingredients into a single shopping list.
///
/// Duplicate ingredients (same `key()`) are summed. Each recipe's quantities are
/// scaled from its authored household size to its own serving target via
/// [`scale_quantity`].
pub(crate) fn merge_ingredients(
    recipe_ingredients: Vec<RecipeIngredients>,
    servings: &Servings<'_>,
) -> Vec<Ingredient> {
    let mut ingredients: HashMap<String, Ingredient> = HashMap::new();
    for (recipe_id, recipe_household_size, list) in recipe_ingredients {
        let target = servings.of(&recipe_id);
        for ingredient in list {
            let scaled = scale_quantity(ingredient.quantity, recipe_household_size, target);
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
    use super::*;
    use imkitchen_types::recipe::{IngredientCategory, IngredientUnit};

    fn ingredient(name: &str, quantity: u32) -> Ingredient {
        Ingredient {
            name: name.to_owned(),
            quantity,
            unit: Some(IngredientUnit::G),
            category: Some(IngredientCategory::Grocery),
        }
    }

    #[test]
    fn merges_each_recipe_at_its_own_servings() {
        // "a" was added for 8 people, "b" has no recorded size → default 2
        // (below its authored 4, so it keeps its own quantity).
        let per_recipe = HashMap::from([("a".to_owned(), 8u16)]);
        let servings = Servings {
            per_recipe: &per_recipe,
            default: 2,
        };
        let merged = merge_ingredients(
            vec![
                ("a".to_owned(), 4, vec![ingredient("rice", 100)]),
                ("b".to_owned(), 4, vec![ingredient("rice", 100)]),
            ],
            &servings,
        );
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].quantity, 200 + 100);
    }

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
