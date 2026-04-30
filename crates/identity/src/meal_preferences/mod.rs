mod update;

use bitcode::{Decode, Encode};
use std::ops::Deref;
pub use update::*;

use evento::{Executor, Projection, metadata::Event};
use imkitchen_types::meal_preferences::{self, Changed};
use imkitchen_types::recipe::DietaryRestriction;

#[derive(Clone)]
pub struct Module<E: Executor>(pub(crate) imkitchen_core::State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = imkitchen_core::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<MealPreferences> {
        let id = id.into();

        create_projection::<E>(&id)
            .execute(&self.executor)
            .await
            .map(|r| {
                r.unwrap_or_else(|| MealPreferences {
                    id,
                    household_size: 4,
                    dietary_restrictions: vec![],
                    cuisine_variety_weight: 1.0,
                    cursor: Default::default(),
                })
            })
    }
}

#[evento::projection(Encode, Decode)]
pub struct MealPreferences {
    pub id: String,
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
}

fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, MealPreferences> {
    Projection::new::<meal_preferences::MealPreferences>(id)
        .handler(handle_updated())
        .safety_check()
}

impl evento::ProjectionAggregator for MealPreferences {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_updated(event: Event<Changed>, data: &mut MealPreferences) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.household_size = event.data.household_size;
    data.dietary_restrictions = event.data.dietary_restrictions;
    data.cuisine_variety_weight = event.data.cuisine_variety_weight;

    Ok(())
}
