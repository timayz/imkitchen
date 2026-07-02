use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0006",
    vec_box![super::m0005::Migration],
    vec_box![crate::origin_framing::m0006::CreateTable]
);
