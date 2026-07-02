use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0007",
    vec_box![super::m0006::Migration],
    vec_box![crate::shopping_recipe::m0007::AddHouseholdSize]
);
