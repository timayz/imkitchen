use evento::{AggregatorName, Executor, LoadResult, SubscribeBuilder};
use imkitchen_db::table::MealPlanRecipe;
use imkitchen_mealplan::{GenerateRequested, GenerationFailed, MealPlan, WeekGenerated};
use imkitchen_recipe::Ingredient;
use imkitchen_shared::{Event, Metadata};
use imkitchen_user::meal_preferences::UserMealPreferences;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;
use std::collections::HashMap;

use crate::{Checked, Generated, Resetted, Shopping, Unchecked};

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<Shopping>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }
}

pub struct ToggleInput {
    pub week: u64,
    pub name: String,
}

impl<E: Executor + Clone> Command<E> {
    pub async fn toggle(
        &self,
        input: ToggleInput,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let user_id = metadata.trigger_by()?;
        let Some(loaded) = self.load(&user_id).await? else {
            imkitchen_shared::user!("ingredient not found");
        };

        let Some(ingredients) = loaded.item.ingredients.get(&input.week) else {
            imkitchen_shared::user!("ingredient not found");
        };

        if !ingredients.contains(&input.name) {
            imkitchen_shared::user!("ingredient not found");
        }

        let checked = loaded
            .item
            .checked
            .get(&input.week)
            .and_then(|v| v.get(&input.name))
            .is_some();

        if checked {
            evento::save_with(loaded)
                .data(&Unchecked {
                    week: input.week,
                    ingredient: input.name,
                })?
                .metadata(metadata)?
                .commit(&self.0)
                .await?;
        } else {
            evento::save_with(loaded)
                .data(&Checked {
                    week: input.week,
                    ingredient: input.name,
                })?
                .metadata(metadata)?
                .commit(&self.0)
                .await?;
        }

        Ok(())
    }
}

impl<E: Executor + Clone> Command<E> {
    pub async fn reset(&self, week: u64, metadata: &Metadata) -> imkitchen_shared::Result<()> {
        let user_id = metadata.trigger_by()?;

        evento::save::<Shopping>(&user_id)
            .data(&Resetted { week })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}

pub fn subscribe_command<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("shopping-command")
        .handler(handle_week_generated())
        .skip::<MealPlan, GenerateRequested>()
        .skip::<MealPlan, GenerationFailed>()
}

#[evento::handler(MealPlan)]
async fn handle_week_generated<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<WeekGenerated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let preferences =
        evento::load_optional::<UserMealPreferences, _>(context.executor, &event.aggregator_id)
            .await?
            .unwrap_or_default();

    let recipe_ids = event
        .data
        .slots
        .iter()
        .flat_map(|slot| {
            let mut ids = vec![slot.main_course.id.to_owned()];

            if let Some(ref r) = slot.appetizer {
                ids.push(r.id.to_owned());
            }

            if let Some(ref r) = slot.dessert {
                ids.push(r.id.to_owned());
            }

            if let Some(ref r) = slot.accompaniment {
                ids.push(r.id.to_owned());
            }

            ids
        })
        .collect::<Vec<_>>();

    let statement = Query::select()
        .columns([MealPlanRecipe::Ingredients, MealPlanRecipe::HouseholdSize])
        .from(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(&event.aggregator_id))
        .and_where(Expr::col(MealPlanRecipe::Id).is_in(recipe_ids))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    let recipe_ingredients =
        sqlx::query_as_with::<_, (imkitchen_db::types::Bincode<Vec<Ingredient>>, u16), _>(
            &sql, values,
        )
        .fetch_all(&pool)
        .await?;

    let mut ingredients = HashMap::new();
    for (recipe_ingredients, household_size) in recipe_ingredients {
        for ingredient in recipe_ingredients.0 {
            let entry = ingredients.entry(ingredient.key()).or_insert(Ingredient {
                name: ingredient.name,
                quantity: 0,
                unit: ingredient.unit,
                category: ingredient.category,
            });

            entry.quantity += ((preferences.item.household_size as u32 * ingredient.quantity
                / household_size as u32) as f64)
                .ceil() as u32;
        }
    }

    evento::save::<Shopping>(&event.aggregator_id)
        .data(&Generated {
            week: event.data.start,
            ingredients: ingredients.values().cloned().collect(),
        })?
        .metadata(&event.metadata)?
        .commit(context.executor)
        .await?;

    Ok(())
}
