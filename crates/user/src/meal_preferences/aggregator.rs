use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::meal_preferences::{Created, Updated};

#[derive(Encode, Decode, Clone, Debug)]
pub struct UserMealPreferences {
    pub household_size: u8,
    pub dietary_restrictions: Vec<String>,
    pub cuisine_variety_weight: f32,
}

impl Default for UserMealPreferences {
    fn default() -> Self {
        Self {
            household_size: 2,
            dietary_restrictions: vec![],
            cuisine_variety_weight: 0.2,
        }
    }
}

#[evento::aggregator]
impl UserMealPreferences {
    async fn handle_created(&mut self, event: Event<Created>) -> anyhow::Result<()> {
        self.household_size = event.data.household_size;
        self.dietary_restrictions = event.data.dietary_restrictions;
        self.cuisine_variety_weight = event.data.cuisine_variety_weight;

        Ok(())
    }

    async fn handle_updated(&mut self, event: Event<Updated>) -> anyhow::Result<()> {
        self.household_size = event.data.household_size;
        self.dietary_restrictions = event.data.dietary_restrictions;
        self.cuisine_variety_weight = event.data.cuisine_variety_weight;

        Ok(())
    }
}
