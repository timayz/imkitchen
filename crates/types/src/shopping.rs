use crate::recipe::Ingredient;

#[evento::aggregator]
pub enum Shopping {
    Checked { ingredient: String },
    Unchecked { ingredient: String },
    Generated { ingredients: Vec<Ingredient> },
}
