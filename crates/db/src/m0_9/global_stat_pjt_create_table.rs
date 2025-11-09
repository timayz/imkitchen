use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::GlobalStatPjt;

pub struct Operation;

fn up_statement() -> TableCreateStatement {
    Table::create()
        .table(GlobalStatPjt::Table)
        .col(
            ColumnDef::new(GlobalStatPjt::Key)
                .string()
                .not_null()
                .string_len(30)
                .primary_key(),
        )
        .col(
            ColumnDef::new(GlobalStatPjt::Value)
                .integer()
                .not_null()
                .default(0),
        )
        .to_owned()
}

fn down_statement() -> TableDropStatement {
    Table::drop().table(GlobalStatPjt::Table).to_owned()
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
