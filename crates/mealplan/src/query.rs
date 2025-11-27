use imkitchen_db::table::MealPlanWeek;
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

use crate::{Slot, Status};

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

#[derive(Clone)]
pub struct Query(pub sqlx::SqlitePool);

impl Query {
    pub async fn find_utc(
        &self,
        week: u64,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<WeekRow>> {
        self.find(OffsetDateTime::from_unix_timestamp(week as i64)?, user_id)
            .await
    }

    pub async fn find(
        &self,
        week: OffsetDateTime,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<WeekRow>> {
        let user_id = user_id.into();
        let statment = sea_query::Query::select()
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

        let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

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
        let start = start.replace_time(time::Time::MIDNIGHT).unix_timestamp() as u64;
        let statment = sea_query::Query::select()
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

        let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, WeekListRow, _>(&sql, values)
            .fetch_all(&self.0)
            .await?)
    }
}
