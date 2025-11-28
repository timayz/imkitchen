use crate::{GenerateRequested, MealPlan, Slot, WeekGenerated};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::MealPlanWeek;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

pub fn subscribe_week<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("mealplan-week")
        .handler(handle_generate_requested())
        .handler(handle_week_generated())
        .handler_check_off()
}

#[evento::handler(MealPlan)]
async fn handle_generate_requested<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<GenerateRequested>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let slots = bincode::encode_to_vec(Vec::<Slot>::default(), config)?;

    let mut statement = Query::insert()
        .into_table(MealPlanWeek::Table)
        .columns([
            MealPlanWeek::UserId,
            MealPlanWeek::Start,
            MealPlanWeek::End,
            MealPlanWeek::Status,
            MealPlanWeek::Slots,
        ])
        .to_owned();

    for (start, end) in &event.data.weeks {
        statement.values_panic([
            event.aggregator_id.to_owned().into(),
            start.to_owned().into(),
            end.to_owned().into(),
            event.data.status.to_string().into(),
            slots.clone().into(),
        ]);
    }

    statement.on_conflict(
        OnConflict::columns([MealPlanWeek::UserId, MealPlanWeek::Start])
            .update_columns([MealPlanWeek::Status, MealPlanWeek::Slots])
            .to_owned(),
    );

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(MealPlan)]
async fn handle_week_generated<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<WeekGenerated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let slots = bincode::encode_to_vec(&event.data.slots, config)?;

    let statment = Query::update()
        .table(MealPlanWeek::Table)
        .values([
            (MealPlanWeek::Status, event.data.status.to_string().into()),
            (MealPlanWeek::Slots, slots.into()),
        ])
        .and_where(Expr::col(MealPlanWeek::UserId).eq(&event.aggregator_id))
        .and_where(Expr::col(MealPlanWeek::Start).eq(event.data.start))
        .to_owned();
    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
