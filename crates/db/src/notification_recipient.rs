use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum NotificationRecipient {
    Table,
    Id,
    Cursor,
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
