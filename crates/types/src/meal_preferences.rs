use crate::recipe::DietaryRestriction;

#[evento::aggregate]
pub enum MealPreferences {
    Changed {
        household_size: u16,
        dietary_restrictions: Vec<DietaryRestriction>,
        cuisine_variety_weight: f32,
    },
}
