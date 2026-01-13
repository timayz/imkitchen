use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::UserLogin;

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
        .col(ColumnDef::new(UserLogin::Username).string().string_len(15))
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
