use evento::{Executor, Projection, ProjectionCursor, Snapshot, cursor, metadata::Event};
use imkitchen_shared::mealplan::{MealPlan, WeekGenerated};

#[derive(Default)]
pub struct LastWeekView {
    pub week: u64,
    pub cursor: String,
}

impl ProjectionCursor for LastWeekView {
    fn set_cursor(&mut self, v: &cursor::Value) {
        self.cursor = v.to_string();
    }

    fn get_cursor(&self) -> cursor::Value {
        self.cursor.to_owned().into()
    }
}

impl Snapshot for LastWeekView {}

pub fn create_projection(id: impl Into<String>) -> Projection<LastWeekView> {
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
