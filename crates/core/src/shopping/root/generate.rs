use evento::{Executor, ProjectionAggregate};
use imkitchen_types::{recipe::RecipeType, shopping::ListGenerated};
use std::collections::HashSet;
use validator::Validate;

use super::{
    meal::{Meal, Pools},
    merge::merge_ingredients,
    pick::{PoolRecipe, Randomize},
};

#[derive(Validate)]
pub struct GenerateList {
    /// How many meals to compose (1..=30), one main course each.
    #[validate(range(min = 1, max = 30))]
    pub count: u8,
    pub household_size: u16,
    pub randomize: Option<Randomize>,
}

impl<E: Executor> super::Module<E> {
    /// Replace the user's list with freshly composed meals. Checks and cooking
    /// statuses are reset.
    ///
    /// Selection rule:
    /// 1. the user's dishes come first: up to `count` of them, shuffled, each
    ///    resolved against the pool (a recipe that was deleted or no longer
    ///    matches the dietary restrictions is dropped; a dish without its main
    ///    is skipped). A dish is a meal as composed, with its sides and sauces;
    /// 2. the remaining meals come from random main courses not used by a
    ///    dish (filtered by dietary restrictions and sized by
    ///    `cuisine_variety_weight` when `randomize` is given, otherwise a
    ///    plain sample). Fewer mains than needed simply yields fewer meals —
    ///    a recipe never appears twice in the list;
    /// 3. every meal is completed with one recipe of each optional course
    ///    type enabled in `randomize.recipe_types` that it lacks (appetizer,
    ///    accompaniment, dessert, beverage, condiment), drawn from that
    ///    type's pool without reuse, so once a pool runs out later meals go
    ///    without that course. Accompaniments are only paired with mains that
    ///    accept one; a dish that already has a side or a sauce keeps its own;
    /// 4. the list is written in meal order — starter, main, sides, dessert,
    ///    drink, sauces — and the merged ingredients are scaled to
    ///    `household_size`.
    pub async fn generate(
        &self,
        input: GenerateList,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        input.validate()?;
        let request_by = request_by.into();
        let randomize = input.randomize.as_ref();
        let count = input.count as usize;
        let dietary_restrictions = randomize
            .map(|opts| opts.dietary_restrictions.to_vec())
            .unwrap_or_default();

        let mut meals = self.dish_meals(&request_by, &dietary_restrictions).await?;
        meals.truncate(count);

        let mut used: HashSet<String> = meals
            .iter()
            .flat_map(|meal| meal.clone().into_ids())
            .collect();

        if meals.len() < count {
            let mains = match randomize {
                Some(opts) => {
                    self.random(
                        &request_by,
                        RecipeType::MainCourse,
                        opts.cuisine_variety_weight,
                        dietary_restrictions,
                    )
                    .await?
                }
                None => {
                    self.sample_recipes(&request_by, RecipeType::MainCourse)
                        .await?
                }
            };

            let missing = count - meals.len();
            let fresh: Vec<PoolRecipe> = mains
                .into_iter()
                .filter(|main| !used.contains(&main.id))
                .take(missing)
                .collect();
            for main in fresh {
                used.insert(main.id.to_owned());
                meals.push(Meal::of_main(main));
            }
        }

        if meals.is_empty() {
            crate::user!("No main course found");
        }

        // One pool query per enabled course for the whole list; a disabled
        // course costs nothing. Recipes already placed by a dish are not drawn
        // again.
        let unused = |recipes: Vec<PoolRecipe>| {
            recipes
                .into_iter()
                .filter(|r| !used.contains(&r.id))
                .collect::<Vec<_>>()
                .into_iter()
        };
        let mut pools = Pools {
            appetizers: unused(
                self.optional_pool(&request_by, RecipeType::Appetizer, randomize)
                    .await?,
            ),
            accompaniments: unused(
                self.optional_pool(&request_by, RecipeType::Accompaniment, randomize)
                    .await?,
            ),
            desserts: unused(
                self.optional_pool(&request_by, RecipeType::Dessert, randomize)
                    .await?,
            ),
            beverages: unused(
                self.optional_pool(&request_by, RecipeType::Beverage, randomize)
                    .await?,
            ),
            condiments: unused(
                self.optional_pool(&request_by, RecipeType::Condiment, randomize)
                    .await?,
            ),
        };

        let mut recipe_ids: Vec<String> = Vec::with_capacity(meals.len() * 4);
        for mut meal in meals {
            meal.fill(&mut pools);
            for id in meal.into_ids() {
                if !recipe_ids.contains(&id) {
                    recipe_ids.push(id);
                }
            }
        }

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(recipe_ids.clone())
            .await?;
        let ingredients = merge_ingredients(recipe_ingredients, input.household_size);

        self.load_or_empty(&request_by)
            .await?
            .write()?
            .event(&ListGenerated {
                recipe_ids,
                ingredients,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
