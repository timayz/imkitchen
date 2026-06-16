use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum UserLogin {
    Table,
    Id,
    Cursor,
    Username,
    Email,
    Role,
    State,
    SubscriptionExpireAt,
    Logins,
}

pub(crate) mod m0001 {
    use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

    use super::UserLogin;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(UserLogin::Table)
            .col(
                ColumnDef::new(UserLogin::Id)
                    .string()
                    .not_null()
                    .string_len(26)
                    .primary_key(),
            )
            .col(ColumnDef::new(UserLogin::Cursor).string().not_null())
            .col(ColumnDef::new(UserLogin::Username).string().string_len(25))
            .col(ColumnDef::new(UserLogin::Logins).blob().not_null())
            .col(
                ColumnDef::new(UserLogin::Role)
                    .string()
                    .not_null()
                    .string_len(15),
            )
            .col(
                ColumnDef::new(UserLogin::State)
                    .string()
                    .not_null()
                    .string_len(15),
            )
            .col(
                ColumnDef::new(UserLogin::SubscriptionExpireAt)
                    .big_integer()
                    .not_null()
                    .default(0),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(UserLogin::Table).to_owned()
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
}

pub(crate) mod m0005 {
    use sea_query::{ColumnDef, DeleteStatement, Query, Table, TableAlterStatement};

    use super::UserLogin;

    pub struct AddEmail;

    fn add_email_column() -> TableAlterStatement {
        Table::alter()
            .table(UserLogin::Table)
            .add_column(
                ColumnDef::new(UserLogin::Email)
                    .string()
                    .not_null()
                    .default(""),
            )
            .to_owned()
    }

    fn drop_email_column() -> TableAlterStatement {
        Table::alter()
            .table(UserLogin::Table)
            .drop_column(UserLogin::Email)
            .to_owned()
    }

    // SQLite has no TRUNCATE — an unqualified DELETE triggers SQLite's
    // truncate optimization (sqlite.org/lang_delete.html). On other backends
    // sea-query will render the equivalent DELETE.
    fn truncate_snapshots() -> DeleteStatement {
        Query::delete().from_table(UserLogin::Table).to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for AddEmail {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let add_column = add_email_column().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(add_column))
                .execute(&mut *connection)
                .await?;

            // Drop existing snapshots so the projection rebuilds with the new
            // Login struct shape (the `logins` blob was bitcoded against the
            // pre-email Login and would fail to decode otherwise).
            let truncate = truncate_snapshots().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(truncate))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let drop_column = drop_email_column().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(drop_column))
                .execute(connection)
                .await?;

            Ok(())
        }
    }
}
