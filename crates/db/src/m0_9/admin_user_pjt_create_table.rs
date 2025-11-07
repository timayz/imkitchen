use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::AdminUserPjt;

pub struct Operation;

fn up_statement() -> TableCreateStatement {
    Table::create()
        .table(AdminUserPjt::Table)
        .col(
            ColumnDef::new(AdminUserPjt::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(AdminUserPjt::Email)
                .string()
                .not_null()
                .string_len(20),
        )
        .col(
            ColumnDef::new(AdminUserPjt::FullName)
                .string()
                .string_len(25),
        )
        .col(
            ColumnDef::new(AdminUserPjt::Username)
                .string()
                .string_len(15),
        )
        .col(
            ColumnDef::new(AdminUserPjt::Status)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(AdminUserPjt::AccountType)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(AdminUserPjt::TotalRecipesCount)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(AdminUserPjt::SharedRecipesCount)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(AdminUserPjt::CreatedAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .to_owned()
}

fn down_statement() -> TableDropStatement {
    Table::drop().table(AdminUserPjt::Table).to_owned()
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
