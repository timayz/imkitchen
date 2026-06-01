use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0004",
    vec_box![super::m0003::Migration],
    vec_box![crate::mealplan_slot::m0004::AddBeverageAndCondiment]
);
