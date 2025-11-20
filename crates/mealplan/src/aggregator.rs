use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::Generated;

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct MealPlan {
    pub user_id: String,
    pub shared: bool,
    pub deleted: bool,
}

#[evento::aggregator]
impl MealPlan {
    async fn handle_generated(&mut self, event: Event<Generated>) -> anyhow::Result<()> {
        self.user_id = event.metadata.trigger_by()?;

        Ok(())
    }
}
