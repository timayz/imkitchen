use crate::{DaySlotRecipe, MealPlan, WeekGenerated};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::{MealPlanRecipe, MealPlanSlot};
use imkitchen_recipe::{Ingredient, Instruction};
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

#[derive(Default, FromRow)]
pub struct MealPlanRecipeRow {
    pub id: String,
    pub name: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: imkitchen_db::types::Bincode<Vec<Ingredient>>,
    pub instructions: imkitchen_db::types::Bincode<Vec<Instruction>>,
    pub advance_prep: String,
}

impl From<&MealPlanRecipeRow> for DaySlotRecipe {
    fn from(value: &MealPlanRecipeRow) -> Self {
        Self {
            id: value.id.to_owned(),
            name: value.name.to_owned(),
            prep_time: value.prep_time.to_owned(),
            cook_time: value.cook_time.to_owned(),
            ingredients: value.ingredients.0.to_owned(),
            instructions: value.instructions.0.to_owned(),
            advance_prep: value.advance_prep.to_owned(),
        }
    }
}

pub fn subscribe_slot<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("mealplan-slot")
        .handler(handle_week_generated())
        .handler_check_off()
}

#[evento::handler(MealPlan)]
async fn handle_week_generated<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<WeekGenerated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
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
        .columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::Name,
            MealPlanRecipe::PrepTime,
            MealPlanRecipe::CookTime,
            MealPlanRecipe::AdvancePrep,
            MealPlanRecipe::Ingredients,
            MealPlanRecipe::Instructions,
        ])
        .from(MealPlanRecipe::Table)
        .and_where(Expr::col(MealPlanRecipe::UserId).eq(&event.aggregator_id))
        .and_where(Expr::col(MealPlanRecipe::Id).is_in(recipe_ids))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    let recipes = sqlx::query_as_with::<_, MealPlanRecipeRow, _>(&sql, values)
        .fetch_all(&pool)
        .await?;

    let mut statement = Query::insert()
        .into_table(MealPlanSlot::Table)
        .columns([
            MealPlanSlot::UserId,
            MealPlanSlot::Day,
            MealPlanSlot::MainCourse,
            MealPlanSlot::Appetizer,
            MealPlanSlot::Accompaniment,
            MealPlanSlot::Dessert,
        ])
        .to_owned();
    let mut has_values = false;
    let user_id = event.aggregator_id.to_owned();
    let config = bincode::config::standard();
    for slot in event.data.slots {
        let Some(main_course): Option<DaySlotRecipe> = recipes
            .iter()
            .find(|r| r.id == slot.main_course.id)
            .map(|r| r.into())
        else {
            continue;
        };

        let main_course = match bincode::encode_to_vec(&main_course, config) {
            Ok(v) => v,
            Err(err) => {
                tracing::error!(sub = "mealplan-slot", err = %err, "failed to encode_to_vec main_course");
                continue;
            }
        };

        let appetizer: Option<DaySlotRecipe> = slot
            .appetizer
            .and_then(|a| recipes.iter().find(|r| r.id == a.id))
            .map(|r| r.into());

        let appetizer = match appetizer {
            Some(appetizer) => match bincode::encode_to_vec(&appetizer, config) {
                Ok(v) => Some(v),
                Err(err) => {
                    tracing::error!(sub = "mealplan-slot", err = %err, "failed to encode_to_vec appetizer");

                    None
                }
            },
            _ => None,
        };

        let accompaniment: Option<DaySlotRecipe> = slot
            .accompaniment
            .and_then(|a| recipes.iter().find(|r| r.id == a.id))
            .map(|r| r.into());

        let accompaniment = match accompaniment {
            Some(accompaniment) => match bincode::encode_to_vec(&accompaniment, config) {
                Ok(v) => Some(v),
                Err(err) => {
                    tracing::error!(sub = "mealplan-slot", err = %err, "failed to encode_to_vec accompaniment");

                    None
                }
            },
            _ => None,
        };

        let dessert: Option<DaySlotRecipe> = slot
            .dessert
            .and_then(|a| recipes.iter().find(|r| r.id == a.id))
            .map(|r| r.into());

        let dessert = match dessert {
            Some(dessert) => match bincode::encode_to_vec(&dessert, config) {
                Ok(v) => Some(v),
                Err(err) => {
                    tracing::error!(sub = "mealplan-slot", err = %err, "failed to encode_to_vec dessert");

                    None
                }
            },
            _ => None,
        };

        statement.values_panic([
            user_id.to_owned().into(),
            slot.day.into(),
            main_course.into(),
            appetizer.into(),
            accompaniment.into(),
            dessert.into(),
        ]);

        has_values = true;
    }

    if !has_values {
        return Ok(());
    }

    statement.on_conflict(
        OnConflict::columns([MealPlanSlot::UserId, MealPlanSlot::Day])
            .update_columns([
                MealPlanSlot::Appetizer,
                MealPlanSlot::MainCourse,
                MealPlanSlot::Accompaniment,
                MealPlanSlot::Dessert,
            ])
            .to_owned(),
    );

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
