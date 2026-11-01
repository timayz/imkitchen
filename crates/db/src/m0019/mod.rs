use sqlx_migrator::vec_box;

pub struct Migration;

// Following a chef: the `chef_follower` list read model and a follower count
// on the chef's `recipe_user_stat` row.
sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0019",
    vec_box![super::m0018::Migration],
    vec_box![
        crate::chef_follower::m0019::CreateTable,
        crate::recipe_user_stat::m0019::AddFollower
    ]
);
