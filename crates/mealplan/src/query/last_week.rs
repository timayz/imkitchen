use bitcode::{Decode, Encode};
use evento::{Executor, Projection, metadata::Event};
use imkitchen_shared::mealplan::{MealPlan, WeekGenerated};

#[evento::projection(Encode, Decode)]
pub struct LastWeekView {
    pub week: u64,
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, LastWeekView> {
    Projection::new::<MealPlan>(id).handler(handle_week_generated())
}

impl<E: Executor> super::Query<E> {
    pub async fn last_week(&self, id: impl Into<String>) -> anyhow::Result<Option<LastWeekView>> {
        load(&self.executor, id).await
    }
}

pub(crate) async fn load<E: Executor>(
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
