use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::ContactGlobalStat;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(ContactGlobalStat::Table)
        .col(
            ColumnDef::new(ContactGlobalStat::Day)
                .string()
                .string_len(20)
                .primary_key(),
        )
        .col(
            ColumnDef::new(ContactGlobalStat::Total)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(ContactGlobalStat::Today)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(ContactGlobalStat::Unread)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(ContactGlobalStat::AvgResponseTime)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(ContactGlobalStat::CreatedAt)
                .big_integer()
                .not_null(),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(ContactGlobalStat::Table).to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateTable {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_table().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_table().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}
