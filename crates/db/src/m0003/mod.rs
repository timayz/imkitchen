use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0003",
    vec_box![super::m0002::Migration],
    vec_box![crate::recipe_user::m0003::DropRatingColumns]
);
