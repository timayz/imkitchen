//! A dish: a named composition of recipes from the user's pool that the
//! kitchen treats as one meal, both when generating the list and when the
//! user adds it by hand.
//!
//! The recipe ids are stored in meal order (starter, main, sides, dessert,
//! drink, sauces). A dish holds exactly one main course, at most one
//! appetizer, dessert and beverage, and any number of accompaniments and
//! condiments; that rule is enforced by the command, not the event.

#[evento::aggregate]
pub enum Dish {
    Created {
        name: String,
        recipe_ids: Vec<String>,
    },
    Changed {
        name: String,
        recipe_ids: Vec<String>,
    },
    Deleted,
}
