use imkitchen_db::table::MealPlanWeek;
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

use crate::{Slot, Status};

#[derive(Default, FromRow)]
pub struct WeekRow {
    pub user_id: String,
    pub week: u64,
    pub slots: imkitchen_db::types::Bincode<Vec<Slot>>,
    pub status: sqlx::types::Text<Status>,
}

#[derive(Clone)]
pub struct Query(pub sqlx::SqlitePool);

impl Query {
    pub async fn find(
        &self,
        week: u64,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<WeekRow>> {
        let user_id = user_id.into();
        let statment = sea_query::Query::select()
            .columns([
                MealPlanWeek::UserId,
                MealPlanWeek::Week,
                MealPlanWeek::Slots,
                MealPlanWeek::Status,
            ])
            .from(MealPlanWeek::Table)
            .and_where(Expr::col(MealPlanWeek::UserId).eq(&user_id))
            .and_where(Expr::col(MealPlanWeek::Week).eq(week))
            .limit(1)
            .to_owned();

        let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, WeekRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}
