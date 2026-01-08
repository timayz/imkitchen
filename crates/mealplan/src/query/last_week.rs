use evento::{Executor, Projection, Snapshot, metadata::Event};

use crate::{MealPlan, WeekGenerated};

// pub async fn find_last_week(
//     &self,
//     user_id: impl Into<String>,
// ) -> imkitchen_shared::Result<Option<u64>> {
//     let id = user_id.into();
//     let statement = Query::select()
//         .columns([MealPlanLastWeek::Start])
//         .from(MealPlanLastWeek::Table)
//         .and_where(Expr::col(MealPlanRecipe::UserId).eq(id))
//         .limit(1)
//         .to_owned();
//
//     let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
//
//     let week = sqlx::query_as_with::<_, (u64,), _>(&sql, values)
//         .fetch_optional(&self.1)
//         .await?;
//
//     Ok(week.map(|w| w.0))
// }
// #[evento::handler(MealPlan)]
// async fn handle_week_generated<E: Executor>(
//     context: &evento::Context<'_, E>,
//     event: Event<WeekGenerated>,
// ) -> anyhow::Result<()> {
//     let pool = context.extract::<sqlx::SqlitePool>();
//
//     let statement = Query::insert()
//         .into_table(MealPlanLastWeek::Table)
//         .columns([MealPlanLastWeek::UserId, MealPlanLastWeek::Start])
//         .values_panic([
//             event.aggregator_id.to_owned().into(),
//             event.data.start.to_owned().into(),
//         ])
//         .on_conflict(
//             OnConflict::columns([MealPlanLastWeek::UserId])
//                 .update_column(MealPlanLastWeek::Start)
//                 .to_owned(),
//         )
//         .to_owned();
//
//     let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
//     sqlx::query_with(&sql, values).execute(&pool).await?;
//
//     Ok(())
// }

#[derive(Default)]
pub struct LastWeekView {
    pub week: u64,
}

impl Snapshot for LastWeekView {}

pub fn create_projection(id: impl Into<String>) -> Projection<LastWeekView> {
    Projection::new::<MealPlan>(id).handler(handle_week_generated())
}

pub async fn load<E: Executor>(
    executor: &E,
    id: impl Into<String>,
) -> anyhow::Result<Option<LastWeekView>> {
    create_projection(id).execute(executor).await
}

#[evento::handler]
async fn handle_week_generated(
    event: Event<WeekGenerated>,
    data: &mut LastWeekView,
) -> anyhow::Result<()> {
    data.week = event.data.start;

    Ok(())
}
