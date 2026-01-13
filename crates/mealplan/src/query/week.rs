use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::MealPlanWeek;
use imkitchen_shared::mealplan::{Slot, WeekGenerated};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

#[derive(Default, FromRow)]
pub struct WeekRow {
    pub user_id: String,
    pub start: u64,
    pub end: u64,
    pub slots: evento::sql_types::Bitcode<Vec<Slot>>,
}

#[derive(Default, FromRow)]
pub struct WeekListRow {
    pub user_id: String,
    pub start: u64,
    pub end: u64,
}

impl<E: Executor> super::Query<E> {
    pub async fn find_from_unix_timestamp(
        &self,
        week: u64,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<WeekRow>> {
        self.find_week(
            OffsetDateTime::from_unix_timestamp(week.try_into()?)?,
            user_id,
        )
        .await
    }

    pub async fn find_week(
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
            ])
            .from(MealPlanWeek::Table)
            .and_where(Expr::col(MealPlanWeek::UserId).eq(&user_id))
            .and_where(Expr::col(MealPlanWeek::Start).eq(week.unix_timestamp()))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, WeekRow, _>(&sql, values)
            .fetch_optional(&self.read_db)
            .await?)
    }

    pub async fn filter_week_last_from(
        &self,
        start: OffsetDateTime,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Vec<WeekListRow>> {
        let user_id = user_id.into();
        let start: u64 = start
            .replace_time(time::Time::from_hms(12, 0, 0)?)
            .unix_timestamp()
            .try_into()?;
        let statement = sea_query::Query::select()
            .columns([MealPlanWeek::UserId, MealPlanWeek::Start, MealPlanWeek::End])
            .from(MealPlanWeek::Table)
            .and_where(Expr::col(MealPlanWeek::UserId).eq(&user_id))
            .and_where(Expr::col(MealPlanWeek::Start).gte(start))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, WeekListRow, _>(&sql, values)
            .fetch_all(&self.read_db)
            .await?)
    }

    pub async fn find_week_last_from(
        &self,
        start: OffsetDateTime,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<WeekRow>> {
        let user_id = user_id.into();
        let start: u64 = start
            .replace_time(time::Time::from_hms(12, 0, 0)?)
            .unix_timestamp()
            .try_into()?;
        let statement = sea_query::Query::select()
            .columns([
                MealPlanWeek::UserId,
                MealPlanWeek::Start,
                MealPlanWeek::End,
                MealPlanWeek::Slots,
            ])
            .from(MealPlanWeek::Table)
            .and_where(Expr::col(MealPlanWeek::UserId).eq(&user_id))
            .and_where(Expr::col(MealPlanWeek::Start).gte(start))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, WeekRow, _>(&sql, values)
            .fetch_optional(&self.read_db)
            .await?)
    }
}

pub fn subscription<E: Executor + Clone>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("mealplan-week").handler(handle_week_generated())
}

#[evento::sub_handler]
async fn handle_week_generated<E: Executor>(
    context: &Context<'_, E>,
    event: Event<WeekGenerated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let slots = bitcode::encode(&event.data.slots);

    let mut statement = Query::insert()
        .into_table(MealPlanWeek::Table)
        .columns([
            MealPlanWeek::UserId,
            MealPlanWeek::Start,
            MealPlanWeek::End,
            MealPlanWeek::Slots,
        ])
        .to_owned();

    statement.values_panic([
        event.aggregator_id.to_owned().into(),
        event.data.start.to_owned().into(),
        event.data.end.to_owned().into(),
        slots.into(),
    ]);

    statement.on_conflict(
        OnConflict::columns([MealPlanWeek::UserId, MealPlanWeek::Start])
            .update_columns([MealPlanWeek::Slots])
            .to_owned(),
    );

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
