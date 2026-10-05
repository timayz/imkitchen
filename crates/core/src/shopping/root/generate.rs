use evento::{Executor, ProjectionAggregate};
use imkitchen_types::{recipe::RecipeType, shopping::ListGenerated};
use validator::Validate;

use super::{merge::merge_ingredients, pick::Randomize};

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
    /// 1. up to `count` main courses from the pool (random, filtered by dietary
    ///    restrictions and sized by `cuisine_variety_weight` when `randomize`
    ///    is given, otherwise a plain sample). Fewer mains than `count` simply
    ///    yields fewer meals — a recipe never appears twice in the list;
    /// 2. each main becomes a meal: it is paired with one recipe of every
    ///    optional course type enabled in `randomize.recipe_types` (appetizer,
    ///    accompaniment, dessert, beverage, condiment), drawn from that type's
    ///    pool without reuse, so once a pool runs out later meals go without
    ///    that course. Accompaniments are only paired with mains that accept
    ///    one;
    /// 3. the list is written in meal order — starter, main, side, dessert,
    ///    drink, sauce — and the merged ingredients are scaled to
    ///    `household_size`.
    pub async fn generate(
        &self,
        input: GenerateList,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        input.validate()?;
        let request_by = request_by.into();
        let randomize = input.randomize.as_ref();

        let mut mains = match randomize {
            Some(opts) => {
                self.random(
                    &request_by,
                    RecipeType::MainCourse,
                    opts.cuisine_variety_weight,
                    opts.dietary_restrictions.to_vec(),
                )
                .await?
            }
            None => {
                self.sample_recipes(&request_by, RecipeType::MainCourse)
                    .await?
            }
        };

        if mains.is_empty() {
            crate::user!("No main course found");
        }

        mains.truncate(input.count as usize);

        // One pool query per enabled course for the whole list; a disabled
        // course costs nothing.
        let mut appetizers = self
            .optional_pool(&request_by, RecipeType::Appetizer, randomize)
            .await?
            .into_iter();
        let mut accompaniments = self
            .optional_pool(&request_by, RecipeType::Accompaniment, randomize)
            .await?
            .into_iter();
        let mut desserts = self
            .optional_pool(&request_by, RecipeType::Dessert, randomize)
            .await?
            .into_iter();
        let mut beverages = self
            .optional_pool(&request_by, RecipeType::Beverage, randomize)
            .await?
            .into_iter();
        let mut condiments = self
            .optional_pool(&request_by, RecipeType::Condiment, randomize)
            .await?
            .into_iter();

        let mut recipe_ids: Vec<String> = Vec::with_capacity(mains.len() * 4);
        for main in mains {
            let accompaniment = if main.accepts_accompaniment {
                accompaniments.next()
            } else {
                None
            };
            for recipe in [
                appetizers.next(),
                Some(main),
                accompaniment,
                desserts.next(),
                beverages.next(),
                condiments.next(),
            ]
            .into_iter()
            .flatten()
            {
                if !recipe_ids.contains(&recipe.id) {
                    recipe_ids.push(recipe.id);
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
