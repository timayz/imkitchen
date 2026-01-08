use crate::Slot;

#[evento::aggregator]
pub enum MealPlan {
    WeekGenerated {
        start: u64,
        end: u64,
        slots: Vec<Slot>,
        household_size: u16,
    },
}
