use crate::recipe::Ingredient;

#[evento::aggregator]
pub enum Shopping {
    Checked {
        week: u64,
        ingredient: String,
    },
    Unchecked {
        week: u64,
        ingredient: String,
    },
    Resetted {
        week: u64,
    },
    Generated {
        week: u64,
        ingredients: Vec<Ingredient>,
    },
}
