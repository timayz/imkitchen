use sqlx_migrator::vec_box;

pub struct Migration;

// evento 2.0.0-alpha.36: projections carry a `tenant`, so the read models
// loaded with `FromRow` store it alongside `cursor` and `aggregate_version`.
sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0017",
    vec_box![super::m0016::Migration],
    vec_box![
        crate::user_login::m0017::AddTenant,
        crate::user_admin::m0017::AddTenant,
        crate::contact_admin::m0017::AddTenant,
        crate::recipe_user::m0017::AddTenant,
        crate::user_invoice_user::m0017::AddTenant,
        crate::notification_recipient::m0017::AddTenant,
    ]
);
