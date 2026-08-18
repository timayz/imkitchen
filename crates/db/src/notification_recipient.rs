use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum NotificationRecipient {
    Table,
    Id,
    Cursor,
    AggregateVersion,
    Email,
    Lang,
    Timezone,
}

pub(crate) mod m0001 {
    use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

    use super::NotificationRecipient;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(NotificationRecipient::Table)
            .col(
                ColumnDef::new(NotificationRecipient::Id)
                    .string()
                    .not_null()
                    .string_len(26)
                    .primary_key(),
            )
            .col(
                ColumnDef::new(NotificationRecipient::Cursor)
                    .string()
                    .not_null()
                    .string_len(26),
            )
            .col(
                ColumnDef::new(NotificationRecipient::Email)
                    .string()
                    .not_null()
                    .string_len(320),
            )
            .col(
                ColumnDef::new(NotificationRecipient::Lang)
                    .string()
                    .not_null()
                    .string_len(10),
            )
            .col(
                ColumnDef::new(NotificationRecipient::Timezone)
                    .string()
                    .not_null()
                    .string_len(50),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(NotificationRecipient::Table).to_owned()
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

pub(crate) mod m0013 {
    use sea_query::{ColumnDef, Table, TableAlterStatement};

    use super::NotificationRecipient;

    pub struct AddAggregateVersion;

    fn add_aggregate_version_column() -> TableAlterStatement {
        Table::alter()
            .table(NotificationRecipient::Table)
            .add_column(
                ColumnDef::new(NotificationRecipient::AggregateVersion)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .to_owned()
    }

    fn drop_aggregate_version_column() -> TableAlterStatement {
        Table::alter()
            .table(NotificationRecipient::Table)
            .drop_column(NotificationRecipient::AggregateVersion)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for AddAggregateVersion {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let add_column =
                add_aggregate_version_column().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(add_column))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let drop_column =
                drop_aggregate_version_column().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(drop_column))
                .execute(connection)
                .await?;

            Ok(())
        }
    }
}
