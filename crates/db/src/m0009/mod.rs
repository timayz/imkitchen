use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0009",
    vec_box![super::m0008::Migration],
    vec_box![crate::recipe_thumbnail::m0009::StripThumbnailBytes]
);
