use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0011",
    vec_box![super::m0010::Migration],
    vec_box![crate::shopping_list::m0011::AddRecipes]
);
