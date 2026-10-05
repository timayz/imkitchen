use evento::{Executor, ProjectionAggregate};
use imkitchen_types::{recipe::RecipeType, shopping::ListGenerated};
use validator::Validate;

use super::{merge::merge_ingredients, pick::Randomize};

#[derive(Validate)]
pub struct GenerateList {
    /// How many main courses to pick (1..=30).
    #[validate(range(min = 1, max = 30))]
    pub count: u8,
    pub household_size: u16,
    pub randomize: Option<Randomize>,
}

impl<E: Executor> super::Module<E> {
    /// Replace the user's list with freshly picked recipes. Checks and cooking
    /// statuses are reset.
    ///
    /// Selection rule:
    /// 1. up to `count` main courses from the pool (random, filtered by dietary
    ///    restrictions and sized by `cuisine_variety_weight` when `randomize`
    ///    is given, otherwise a plain sample). Fewer mains than `count` simply
    ///    yields a shorter list — a flat list never repeats a recipe;
    /// 2. for each optional course type enabled in `randomize.recipe_types`
    ///    (appetizer, accompaniment, dessert, beverage, condiment, in that
    ///    order), up to `ceil(count / 2)` recipes of that type. Accompaniments
    ///    are skipped unless at least one picked main accepts one;
    /// 3. the merged ingredient list is scaled to `household_size`.
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
        let accepts_accompaniment = mains.iter().any(|r| r.accepts_accompaniment);
        let per_type = (input.count as usize).div_ceil(2);

        let mut recipe_ids: Vec<String> = mains.into_iter().map(|r| r.id).collect();

        for recipe_type in [
            RecipeType::Appetizer,
            RecipeType::Accompaniment,
            RecipeType::Dessert,
            RecipeType::Beverage,
            RecipeType::Condiment,
        ] {
            if recipe_type == RecipeType::Accompaniment && !accepts_accompaniment {
                continue;
            }

            let picked = self
                .optional_pool(&request_by, recipe_type, randomize)
                .await?;

            for recipe in picked.into_iter().take(per_type) {
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
