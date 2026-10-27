//! One meal of a generated list: a main course with its optional courses.
//! Dishes pre-fill a meal; the generator fills what is missing from the
//! random pools.

use evento::Executor;
use imkitchen_db::mealplan_recipe::MealPlanRecipe;
use imkitchen_types::recipe::{DietaryRestriction, RecipeType};
use rand::seq::SliceRandom;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use std::collections::HashMap;

use super::pick::{POOL_COLUMNS, PoolRecipe, restrict_dietary};

#[derive(Clone, Debug, Default)]
pub struct Meal {
    pub appetizer: Option<PoolRecipe>,
    pub main: Option<PoolRecipe>,
    pub accompaniments: Vec<PoolRecipe>,
    pub dessert: Option<PoolRecipe>,
    pub beverage: Option<PoolRecipe>,
    pub condiments: Vec<PoolRecipe>,
}

/// The random pools, one per optional course, drawn from without reuse.
pub struct Pools {
    pub appetizers: std::vec::IntoIter<PoolRecipe>,
    pub accompaniments: std::vec::IntoIter<PoolRecipe>,
    pub desserts: std::vec::IntoIter<PoolRecipe>,
    pub beverages: std::vec::IntoIter<PoolRecipe>,
    pub condiments: std::vec::IntoIter<PoolRecipe>,
}

impl Meal {
    pub fn of_main(main: PoolRecipe) -> Self {
        Meal {
            main: Some(main),
            ..Default::default()
        }
    }

    /// Place a recipe by its course. A second appetizer, main, dessert or
    /// beverage replaces the first; sides and sauces accumulate.
    pub fn put(&mut self, recipe: PoolRecipe) {
        match recipe.recipe_type() {
            RecipeType::Appetizer => self.appetizer = Some(recipe),
            RecipeType::MainCourse => self.main = Some(recipe),
            RecipeType::Accompaniment => self.accompaniments.push(recipe),
            RecipeType::Dessert => self.dessert = Some(recipe),
            RecipeType::Beverage => self.beverage = Some(recipe),
            RecipeType::Condiment => self.condiments.push(recipe),
        }
    }

    /// Draw the courses this meal lacks from the pools: an empty pool (the
    /// course is disabled or exhausted) leaves the slot empty. A side is
    /// only drawn for a main that accepts one.
    pub fn fill(&mut self, pools: &mut Pools) {
        if self.appetizer.is_none() {
            self.appetizer = pools.appetizers.next();
        }
        let accepts_accompaniment = self.main.as_ref().is_some_and(|m| m.accepts_accompaniment);
        if self.accompaniments.is_empty()
            && accepts_accompaniment
            && let Some(side) = pools.accompaniments.next()
        {
            self.accompaniments.push(side);
        }
        if self.dessert.is_none() {
            self.dessert = pools.desserts.next();
        }
        if self.beverage.is_none() {
            self.beverage = pools.beverages.next();
        }
        if self.condiments.is_empty()
            && let Some(sauce) = pools.condiments.next()
        {
            self.condiments.push(sauce);
        }
    }

    /// Recipe ids in meal order: starter, main, sides, dessert, drink, sauces.
    pub fn into_ids(self) -> Vec<String> {
        self.appetizer
            .into_iter()
            .chain(self.main)
            .chain(self.accompaniments)
            .chain(self.dessert)
            .chain(self.beverage)
            .chain(self.condiments)
            .map(|r| r.id)
            .collect()
    }
}

impl<E: Executor> super::Module<E> {
    /// The user's dishes as meals, shuffled. Each dish's recipes are resolved
    /// against the pool under the dietary restrictions: a recipe that no
    /// longer qualifies (deleted, unsaved, or not matching the restrictions)
    /// is dropped, and a dish whose main course is dropped is skipped.
    pub(crate) async fn dish_meals(
        &self,
        user_id: &str,
        dietary_restrictions: &[DietaryRestriction],
    ) -> anyhow::Result<Vec<Meal>> {
        let dishes = crate::dish::query::list_rows(&self.read_db, user_id).await?;
        if dishes.is_empty() {
            return Ok(vec![]);
        }

        let ids: Vec<String> = dishes
            .iter()
            .flat_map(|d| d.recipe_ids.0.iter().cloned())
            .collect();

        let mut statement = Query::select()
            .columns(POOL_COLUMNS)
            .from(MealPlanRecipe::Table)
            .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id))
            .and_where(Expr::col(MealPlanRecipe::Id).is_in(ids))
            .and_where(Expr::col(MealPlanRecipe::Name).not_equals(""))
            .to_owned();
        restrict_dietary(&mut statement, dietary_restrictions);

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        let recipes: HashMap<String, PoolRecipe> =
            sqlx::query_as_with::<_, PoolRecipe, _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_all(&self.read_db)
                .await?
                .into_iter()
                .map(|r| (r.id.to_owned(), r))
                .collect();

        let mut meals: Vec<Meal> = dishes
            .into_iter()
            .filter_map(|dish| {
                let mut meal = Meal::default();
                for id in dish.recipe_ids.0.iter() {
                    if let Some(recipe) = recipes.get(id) {
                        meal.put(recipe.clone());
                    }
                }
                meal.main.is_some().then_some(meal)
            })
            .collect();

        let mut rng = rand::rng();
        meals.shuffle(&mut rng);

        Ok(meals)
    }
}

#[cfg(test)]
mod tests {
    use super::{Meal, Pools};
    use crate::shopping::PoolRecipe;
    use imkitchen_types::recipe::RecipeType;

    fn recipe(id: &str, recipe_type: RecipeType, accepts_accompaniment: bool) -> PoolRecipe {
        PoolRecipe {
            id: id.to_owned(),
            name: id.to_owned(),
            recipe_type: sqlx::types::Text(recipe_type),
            accepts_accompaniment,
        }
    }

    fn pools(sides: &[&str], desserts: &[&str]) -> Pools {
        let of = |ids: &[&str], t: RecipeType| {
            ids.iter()
                .map(|id| recipe(id, t.clone(), false))
                .collect::<Vec<_>>()
                .into_iter()
        };
        Pools {
            appetizers: of(&[], RecipeType::Appetizer),
            accompaniments: of(sides, RecipeType::Accompaniment),
            desserts: of(desserts, RecipeType::Dessert),
            beverages: of(&[], RecipeType::Beverage),
            condiments: of(&[], RecipeType::Condiment),
        }
    }

    #[test]
    fn a_dish_keeps_its_sides_and_only_fills_missing_courses() {
        let mut meal = Meal::default();
        meal.put(recipe("sauce", RecipeType::Condiment, false));
        meal.put(recipe("main", RecipeType::MainCourse, true));
        meal.put(recipe("side-a", RecipeType::Accompaniment, false));
        meal.put(recipe("side-b", RecipeType::Accompaniment, false));

        let mut pools = pools(&["random-side"], &["cake"]);
        meal.fill(&mut pools);

        assert_eq!(
            meal.into_ids(),
            vec!["main", "side-a", "side-b", "cake", "sauce"]
        );
        // The random side was not consumed.
        assert_eq!(pools.accompaniments.len(), 1);
    }

    #[test]
    fn a_side_is_drawn_only_for_mains_that_accept_one() {
        let mut meal = Meal::of_main(recipe("soup", RecipeType::MainCourse, false));
        let mut pools = pools(&["bread"], &[]);
        meal.fill(&mut pools);
        assert_eq!(meal.into_ids(), vec!["soup"]);

        let mut meal = Meal::of_main(recipe("steak", RecipeType::MainCourse, true));
        meal.fill(&mut pools);
        assert_eq!(meal.into_ids(), vec!["steak", "bread"]);
    }
}
