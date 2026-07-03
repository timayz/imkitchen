use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum UserAdmin {
    Table,
    Id,
    Cursor,
    Email,
    FullName,
    Username,
    State,
    Role,
    SubscriptionExpireAt,
    TotalRecipesCount,
    SharedRecipesCount,
    TotalActiveCount,
    CreatedAt,
}

#[derive(Iden, Clone)]
pub enum UserAdminFts {
    Table,
    Id,
    Email,
    Username,
    Rank,
}

pub(crate) mod m0001 {
    use sea_query::{
        ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
        TableDropStatement,
    };

    use super::UserAdmin;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(UserAdmin::Table)
            .col(
                ColumnDef::new(UserAdmin::Id)
                    .string()
                    .not_null()
                    .string_len(26)
                    .primary_key(),
            )
            .col(ColumnDef::new(UserAdmin::Cursor).string().not_null())
            .col(
                ColumnDef::new(UserAdmin::Email)
                    .string()
                    .not_null()
                    .string_len(320),
            )
            .col(ColumnDef::new(UserAdmin::FullName).string().string_len(25))
            .col(ColumnDef::new(UserAdmin::Username).string().string_len(25))
            .col(
                ColumnDef::new(UserAdmin::State)
                    .string()
                    .not_null()
                    .string_len(15),
            )
            .col(
                ColumnDef::new(UserAdmin::Role)
                    .string()
                    .not_null()
                    .string_len(15),
            )
            .col(
                ColumnDef::new(UserAdmin::TotalRecipesCount)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(UserAdmin::SharedRecipesCount)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(UserAdmin::TotalActiveCount)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(UserAdmin::SubscriptionExpireAt)
                    .big_integer()
                    .default(0),
            )
            .col(
                ColumnDef::new(UserAdmin::CreatedAt)
                    .big_integer()
                    .not_null(),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(UserAdmin::Table).to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateTable {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_table().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_table().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    pub struct CreateIdx1;

    fn create_idx_1() -> IndexCreateStatement {
        Index::create()
            .name("idx_user_list_D0nmGG")
            .table(UserAdmin::Table)
            .col(UserAdmin::State)
            .to_owned()
    }

    fn drop_idx_1() -> IndexDropStatement {
        Index::drop()
            .name("idx_user_list_D0nmGG")
            .table(UserAdmin::Table)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx1 {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_idx_1().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_idx_1().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    pub struct CreateIdx2;

    fn create_idx_2() -> IndexCreateStatement {
        Index::create()
            .name("idx_user_list_rmnGYD")
            .table(UserAdmin::Table)
            .col(UserAdmin::Role)
            .to_owned()
    }

    fn drop_idx_2() -> IndexDropStatement {
        Index::drop()
            .name("idx_user_list_rmnGYD")
            .table(UserAdmin::Table)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx2 {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_idx_2().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_idx_2().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    pub struct CreateFTSTable;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateFTSTable {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query(
                r#"
CREATE VIRTUAL TABLE user_admin_fts USING fts5(id, email, username);

CREATE TRIGGER user_admin_insert AFTER
INSERT ON user_admin BEGIN
INSERT INTO user_admin_fts (id, email, username)
VALUES (new.id, new.email, COALESCE(new.username, '')); END;

CREATE TRIGGER user_admin_update AFTER
UPDATE on user_admin BEGIN
UPDATE user_admin_fts SET username = COALESCE(new.username, '')
WHERE user_admin_fts = new.id; END;

CREATE TRIGGER user_admin_delete AFTER
DELETE ON user_admin BEGIN
DELETE FROM user_admin_fts WHERE id = old.id; END;
            "#,
            )
            .execute(connection)
            .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query(
                r#"
DROP TRIGGER user_admin_insert;
DROP TRIGGER user_admin_update;
DROP TRIGGER user_admin_delete;
DROP TABLE user_admin_fts;
            "#,
            )
            .execute(connection)
            .await?;

            Ok(())
        }
    }
}

pub(crate) mod m0007 {
    pub struct ResetPremiumSmear;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for ResetPremiumSmear {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            // Before evento 2.0.0-alpha.24, the background `user-query` projection read the
            // co-keyed `Subscription` events (`LifePremiumToggled` / `StripePaymentIntentSucceeded`)
            // unscoped (`by_type`) and smeared one user's premium expiry across every admin row.
            // Delete the rows rather than resetting the per-row `cursor`. An empty cursor is not a
            // valid encoding (reload fails to decode it), and a surviving row would carry its
            // smeared `subscription_expire_at` forward via `snapshot.unwrap_or_default()` — a
            // no-event victim has nothing in the replay to reset the field. Deleting makes each
            // reload start from `Default` (expiry 0) and re-apply only that user's own (now
            // id-scoped) subscription events. The `user_admin_delete` trigger clears the FTS rows;
            // the insert trigger refills them as the projection rebuilds.
            sqlx::query("DELETE FROM user_admin")
                .execute(&mut *connection)
                .await?;

            // Rewind the projection's subscription so the worker replays the whole stream and
            // rebuilds `user_admin` on next start (same mechanism as the m0007 recipe backfill).
            sqlx::query("UPDATE subscriber SET cursor = NULL WHERE key = 'user-query'")
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            _connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            // One-way data repair: the projection rebuilds itself on replay, nothing to revert.
            Ok(())
        }
    }
}

pub(crate) mod m0008 {
    pub struct SyncEmailFts;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for SyncEmailFts {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            // The original `user_admin_update` trigger only re-synced `username` to the FTS index,
            // so an admin-changed email stayed searchable only by its old value. Recreate the
            // trigger to also propagate `email`.
            sqlx::query(
                r#"
DROP TRIGGER user_admin_update;

CREATE TRIGGER user_admin_update AFTER
UPDATE on user_admin BEGIN
UPDATE user_admin_fts SET email = new.email, username = COALESCE(new.username, '')
WHERE user_admin_fts = new.id; END;
            "#,
            )
            .execute(connection)
            .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query(
                r#"
DROP TRIGGER user_admin_update;

CREATE TRIGGER user_admin_update AFTER
UPDATE on user_admin BEGIN
UPDATE user_admin_fts SET username = COALESCE(new.username, '')
WHERE user_admin_fts = new.id; END;
            "#,
            )
            .execute(connection)
            .await?;

            Ok(())
        }
    }
}
