use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::{MealPlanRecipe, MealPlanSlot};
use imkitchen_shared::mealplan::{DaySlotRecipe, WeekGenerated};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};
use time::OffsetDateTime;

#[derive(Default, FromRow)]
pub struct MealPlanRecipeRow {
    pub id: String,
    pub name: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub advance_prep: String,
}

impl From<&MealPlanRecipeRow> for DaySlotRecipe {
    fn from(value: &MealPlanRecipeRow) -> Self {
        Self {
            id: value.id.to_owned(),
            name: value.name.to_owned(),
            prep_time: value.prep_time.to_owned(),
            cook_time: value.cook_time.to_owned(),
            advance_prep: value.advance_prep.to_owned(),
        }
    }
}

#[derive(Default, FromRow)]
pub struct SlotRow {
    pub day: u64,
    pub main_course: evento::sql_types::Bitcode<DaySlotRecipe>,
    pub appetizer: Option<evento::sql_types::Bitcode<DaySlotRecipe>>,
    pub accompaniment: Option<evento::sql_types::Bitcode<DaySlotRecipe>>,
    pub dessert: Option<evento::sql_types::Bitcode<DaySlotRecipe>>,
}

pub async fn next_slot_from(
    pool: &SqlitePool,
    day: OffsetDateTime,
    user_id: impl Into<String>,
) -> anyhow::Result<Option<SlotRow>> {
    let user_id = user_id.into();
    let statement = sea_query::Query::select()
        .columns([
            MealPlanSlot::Day,
            MealPlanSlot::MainCourse,
            MealPlanSlot::Appetizer,
            MealPlanSlot::Accompaniment,
            MealPlanSlot::Dessert,
        ])
        .from(MealPlanSlot::Table)
        .and_where(Expr::col(MealPlanSlot::UserId).eq(&user_id))
        .and_where(Expr::col(MealPlanSlot::Day).gte(day.unix_timestamp()))
        .order_by_expr(Expr::col(MealPlanSlot::Day), sea_query::Order::Asc)
        .limit(1)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with::<_, SlotRow, _>(&sql, values)
        .fetch_optional(pool)
        .await?)
}

pub async fn next_prep_remiders_from(
    pool: &SqlitePool,
    day: u64,
    user_id: impl Into<String>,
) -> anyhow::Result<Option<Vec<DaySlotRecipe>>> {
    let day = OffsetDateTime::from_unix_timestamp(day.try_into()?)?;
    let next_day = day + time::Duration::days(1);
    let Some(slot) = next_slot_from(pool, next_day, user_id).await? else {
        return Ok(None);
    };

    let mut remiders = vec![];

    if !slot.main_course.advance_prep.is_empty() {
        remiders.push(slot.main_course.0);
    }

    let recipe = slot.appetizer.and_then(|r| {
        if !r.advance_prep.is_empty() {
            Some(r.0)
        } else {
            None
        }
    });

    if let Some(recipe) = recipe {
        remiders.push(recipe);
    }

    let recipe = slot.accompaniment.and_then(|r| {
        if !r.advance_prep.is_empty() {
            Some(r.0)
        } else {
            None
        }
    });

    if let Some(recipe) = recipe {
        remiders.push(recipe);
    }

    let recipe = slot.dessert.and_then(|r| {
        if !r.advance_prep.is_empty() {
            Some(r.0)
        } else {
            None
        }
    });

    if let Some(recipe) = recipe {
        remiders.push(recipe);
    }

    if remiders.is_empty() {
        return Ok(None);
    }

    Ok(Some(remiders))
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("mealplan-slot").handler(handle_week_generated())
}

#[evento::sub_handler]
async fn handle_week_generated<E: Executor>(
    context: &Context<'_, E>,
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
    for slot in event.data.slots {
        let Some(main_course): Option<DaySlotRecipe> = recipes
            .iter()
            .find(|r| r.id == slot.main_course.id)
            .map(|r| r.into())
        else {
            continue;
        };

        let main_course = bitcode::encode(&main_course);

        let appetizer: Option<DaySlotRecipe> = slot
            .appetizer
            .and_then(|a| recipes.iter().find(|r| r.id == a.id))
            .map(|r| r.into());

        let appetizer = appetizer.map(|r| bitcode::encode(&r));

        let accompaniment: Option<DaySlotRecipe> = slot
            .accompaniment
            .and_then(|a| recipes.iter().find(|r| r.id == a.id))
            .map(|r| r.into());

        let accompaniment = accompaniment.map(|r| bitcode::encode(&r));

        let dessert: Option<DaySlotRecipe> = slot
            .dessert
            .and_then(|a| recipes.iter().find(|r| r.id == a.id))
            .map(|r| r.into());

        let dessert = dessert.map(|r| bitcode::encode(&r));

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
