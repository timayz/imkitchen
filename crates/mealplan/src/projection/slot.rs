use imkitchen_recipe::{Ingredient, Instruction};
use sqlx::prelude::FromRow;

#[derive(Default, FromRow)]
pub struct MealPlanRecipeRow {
    pub id: String,
    pub name: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: imkitchen_db::types::Bincode<Vec<Ingredient>>,
    pub instructions: imkitchen_db::types::Bincode<Vec<Instruction>>,
    pub advance_prep: String,
}
