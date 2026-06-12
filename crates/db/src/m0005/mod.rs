use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0005",
    vec_box![super::m0004::Migration],
    vec_box![
        crate::user_login::m0005::AddEmail,
        crate::recipe_user::m0005::AddSlug,
        crate::recipe_owner::m0005::CreateTable,
        crate::recipe_owner::m0005::RemoveLegacyShareSubscriber,
        crate::recipe_user_stat::m0005::Rebuild,
    ]
);
