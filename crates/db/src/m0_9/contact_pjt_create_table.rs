use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::ContactPjt;

pub struct Operation;

fn up_statement() -> TableCreateStatement {
    Table::create()
        .table(ContactPjt::Table)
        .col(
            ColumnDef::new(ContactPjt::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(ContactPjt::Name)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(ContactPjt::Email)
                .string()
                .not_null()
                .string_len(320),
        )
        .col(
            ColumnDef::new(ContactPjt::Status)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(ContactPjt::Subject)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(ContactPjt::Message)
                .string()
                .not_null()
                .string_len(2000),
        )
        .col(
            ColumnDef::new(ContactPjt::CreatedAt)
                .big_integer()
                .not_null(),
        )
        .to_owned()
}

fn down_statement() -> TableDropStatement {
    Table::drop().table(ContactPjt::Table).to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for Operation {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = up_statement().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = down_statement().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}
