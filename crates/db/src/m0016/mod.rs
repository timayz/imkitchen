use sqlx_migrator::vec_box;

pub struct Migration;

// Dishes: user-composed meals used by list generation and manual adds.
sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0016",
    vec_box![super::m0015::Migration],
    vec_box![
        crate::dish::m0016::CreateTable,
        crate::dish::m0016::CreateIdx1,
    ]
);
