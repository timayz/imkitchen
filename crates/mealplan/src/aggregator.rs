use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::{GenerateRequested, Status, WeekGenerated};

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct MealPlan {
    pub status: Status,
}

#[evento::aggregator]
impl MealPlan {
    async fn handle_generate_requested(
        &mut self,
        event: Event<GenerateRequested>,
    ) -> anyhow::Result<()> {
        self.status = event.data.status;

        Ok(())
    }

    async fn handle_week_generated(&mut self, event: Event<WeekGenerated>) -> anyhow::Result<()> {
        self.status = event.data.status;

        Ok(())
    }
}
