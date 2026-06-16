use crate::recipe::Ingredient;

#[evento::aggregate]
pub enum Shopping {
    Checked {
        ingredient: String,
    },
    Unchecked {
        ingredient: String,
    },
    Generated {
        ingredients: Vec<Ingredient>,
        from_date: u64,
        days: u8,
    },
}
