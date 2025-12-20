use bincode::{Decode, Encode};
use imkitchen_shared::Event;

#[derive(Encode, Decode, Clone, Debug)]
pub struct RecipeRating {}

impl Default for RecipeRating {
    fn default() -> Self {
        Self {}
    }
}

#[evento::aggregator]
impl RecipeRating {
    // async fn handle_created(&mut self, event: Event<Created>) -> anyhow::Result<()> {
    //     self.household_size = event.data.household_size;
    //     self.dietary_restrictions = event.data.dietary_restrictions;
    //     self.cuisine_variety_weight = event.data.cuisine_variety_weight;
    //
    //     Ok(())
    // }
}
