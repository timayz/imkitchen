use sea_query::{ColumnDef, Expr, Table, TableCreateStatement, TableDropStatement};

use crate::table::User;

pub struct Operation;

fn create_user_table_statement() -> TableCreateStatement {
    Table::create()
        .table(User::Table)
        .col(
            ColumnDef::new(User::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(User::Email)
                .string()
                .not_null()
                .string_len(20),
        )
        .col(
            ColumnDef::new(User::Role)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(User::CreatedAt)
                .timestamp_with_time_zone()
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .to_owned()
}

fn drop_user_table_statement() -> TableDropStatement {
    Table::drop().table(User::Table).to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for Operation {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_user_table_statement().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_user_table_statement().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}
