use evento::{Executor, ProjectionAggregate};
use imkitchen_types::{
    meal_preferences::MealSources,
    recipe::RecipeType,
    shopping::{DishPlaced, ListGenerated, ServingsSet},
};
use std::collections::{HashMap, HashSet};
use validator::Validate;

use super::{
    MAX_HOUSEHOLD_SIZE,
    meal::{Meal, Pools},
    merge::{Servings, merge_ingredients},
    pick::{PoolRecipe, Randomize},
};

#[derive(Validate)]
pub struct GenerateList {
    /// How many meals to compose (1..=30), one main course each.
    #[validate(range(min = 1, max = 30))]
    pub count: u8,
    /// People to cook for; every generated recipe is scaled to it.
    #[validate(range(min = 1, max = MAX_HOUSEHOLD_SIZE))]
    pub household_size: u16,
    /// Dishes, random main courses or both (the settings toggles).
    pub sources: MealSources,
    pub randomize: Option<Randomize>,
}

impl<E: Executor> super::Module<E> {
    /// Replace the user's list with freshly composed meals. Checks and cooking
    /// statuses are reset.
    ///
    /// Selection rule, each step only when `sources` enables it:
    /// 1. the user's dishes come first: up to `count` of them, shuffled, each
    ///    resolved against the pool (a recipe that was deleted or no longer
    ///    matches the dietary restrictions is dropped; a dish without its main
    ///    is skipped). A dish is a meal as composed, with its sides and sauces;
    /// 2. the remaining meals come from random main courses not used by a
    ///    dish (filtered by dietary restrictions and sized by
    ///    `cuisine_variety_weight` when `randomize` is given, otherwise a
    ///    plain sample). Fewer mains (or dishes) than needed simply yields
    ///    fewer meals — a recipe never appears twice in the list. No meal at
    ///    all is a user error;
    /// 3. every meal is completed with one recipe of each optional course
    ///    type enabled in `randomize.recipe_types` that it lacks (appetizer,
    ///    accompaniment, dessert, beverage, condiment), drawn from that
    ///    type's pool without reuse, so once a pool runs out later meals go
    ///    without that course. Accompaniments are only paired with mains that
    ///    accept one; a dish that already has a side or a sauce keeps its own;
    /// 4. the list is written in meal order — starter, main, sides, dessert,
    ///    drink, sauces — and the merged ingredients are scaled to
    ///    `household_size`, which is recorded for every recipe (`ServingsSet`);
    /// 5. each dish meal's own recipes (as composed, before the fill) are
    ///    recorded as a group (`DishPlaced`) so the kitchen shows the dish as
    ///    one row.
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

        let mut meals = if input.sources.dishes() {
            self.dish_meals(&request_by, &dietary_restrictions).await?
        } else {
            vec![]
        };
        meals.truncate(count);

        let mut used: HashSet<String> = meals.iter().flat_map(Meal::ids).collect();

        if meals.len() < count && input.sources.main_courses() {
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
            if input.sources.main_courses() {
                crate::user!("No main course found");
            }
            crate::user!("No dish found");
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

        // The dish's own recipes, before random courses are filled in around it.
        let placed: Vec<DishPlaced> = meals
            .iter()
            .filter_map(|meal| {
                meal.dish_id.clone().map(|dish_id| DishPlaced {
                    dish_id,
                    recipe_ids: meal.ids(),
                })
            })
            .collect();

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
        let ingredients = merge_ingredients(
            recipe_ingredients,
            &Servings {
                per_recipe: &HashMap::new(),
                default: input.household_size,
            },
        );

        let shopping = self.load_or_empty(&request_by).await?;
        let mut builder = shopping.write()?.requested_by(request_by).to_owned();
        builder.event(&ListGenerated {
            recipe_ids: recipe_ids.clone(),
            ingredients,
        });
        builder.event(&ServingsSet {
            recipe_ids,
            household_size: input.household_size,
        });
        for dish in &placed {
            builder.event(dish);
        }
        builder.commit(&self.executor).await?;

        Ok(())
    }
}
