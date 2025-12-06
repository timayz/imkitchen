use crate::{GenerateRequested, MealPlan, Slot, Status, WeekGenerated};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::MealPlanWeek;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

#[derive(Default, FromRow)]
pub struct WeekRow {
    pub user_id: String,
    pub start: u64,
    pub end: u64,
    pub slots: imkitchen_db::types::Bincode<Vec<Slot>>,
    pub status: sqlx::types::Text<Status>,
}

#[derive(Default, FromRow)]
pub struct WeekListRow {
    pub user_id: String,
    pub start: u64,
    pub end: u64,
    pub status: sqlx::types::Text<Status>,
}

impl super::Query {
    pub async fn find_from_unix_timestamp(
        &self,
        week: u64,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<WeekRow>> {
        self.find(
            OffsetDateTime::from_unix_timestamp(week.try_into()?)?,
            user_id,
        )
        .await
    }

    pub async fn find(
        &self,
        week: OffsetDateTime,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<WeekRow>> {
        let user_id = user_id.into();
        let statement = sea_query::Query::select()
            .columns([
                MealPlanWeek::UserId,
                MealPlanWeek::Start,
                MealPlanWeek::End,
                MealPlanWeek::Slots,
                MealPlanWeek::Status,
            ])
            .from(MealPlanWeek::Table)
            .and_where(Expr::col(MealPlanWeek::UserId).eq(&user_id))
            .and_where(Expr::col(MealPlanWeek::Start).eq(week.unix_timestamp()))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, WeekRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }

    pub async fn filter_last_from(
        &self,
        start: OffsetDateTime,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Vec<WeekListRow>> {
        let user_id = user_id.into();
        let start: u64 = start
            .replace_time(time::Time::MIDNIGHT)
            .unix_timestamp()
            .try_into()?;
        let statement = sea_query::Query::select()
            .columns([
                MealPlanWeek::UserId,
                MealPlanWeek::Start,
                MealPlanWeek::End,
                MealPlanWeek::Status,
            ])
            .from(MealPlanWeek::Table)
            .and_where(Expr::col(MealPlanWeek::UserId).eq(&user_id))
            .and_where(Expr::col(MealPlanWeek::Start).gte(start))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, WeekListRow, _>(&sql, values)
            .fetch_all(&self.0)
            .await?)
    }

    pub async fn find_last_from(
        &self,
        start: OffsetDateTime,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<WeekRow>> {
        let user_id = user_id.into();
        let start: u64 = start
            .replace_time(time::Time::MIDNIGHT)
            .unix_timestamp()
            .try_into()?;
        let statement = sea_query::Query::select()
            .columns([
                MealPlanWeek::UserId,
                MealPlanWeek::Start,
                MealPlanWeek::End,
                MealPlanWeek::Slots,
                MealPlanWeek::Status,
            ])
            .from(MealPlanWeek::Table)
            .and_where(Expr::col(MealPlanWeek::UserId).eq(&user_id))
            .and_where(Expr::col(MealPlanWeek::Start).gte(start))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, WeekRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}

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

    let statement = Query::update()
        .table(MealPlanWeek::Table)
        .values([
            (MealPlanWeek::Status, event.data.status.to_string().into()),
            (MealPlanWeek::Slots, slots.into()),
        ])
        .and_where(Expr::col(MealPlanWeek::UserId).eq(&event.aggregator_id))
        .and_where(Expr::col(MealPlanWeek::Start).eq(event.data.start))
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
