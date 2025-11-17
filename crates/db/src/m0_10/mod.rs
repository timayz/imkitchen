mod recipe_pjt_create_cuisine_type_idx;
mod recipe_pjt_create_recipe_type_idx;
mod recipe_pjt_create_table;
mod recipe_pjt_create_user_id_idx;

use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "main",
    "m0_10",
    vec_box![],
    vec_box![
        recipe_pjt_create_table::Operation,
        recipe_pjt_create_recipe_type_idx::Operation,
        recipe_pjt_create_cuisine_type_idx::Operation,
        recipe_pjt_create_user_id_idx::Operation,
    ]
);
