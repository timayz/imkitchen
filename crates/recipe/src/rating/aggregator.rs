use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, Debug, Default)]
pub struct RecipeRating {}

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
