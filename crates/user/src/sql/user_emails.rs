use sea_query::{ColumnDef, Expr, Iden, Table, TableCreateStatement, TableDropStatement};
use sqlx_migrator::{Operation, vec_box};

#[derive(Iden, Clone)]
pub enum UserEmail {
    Table,
    Email,
    CreatedAt,
}

pub struct CreateUserEmailTable;

fn create_user_email_table_statement() -> TableCreateStatement {
    Table::create()
        .table(UserEmail::Table)
        .col(
            ColumnDef::new(UserEmail::Email)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(UserEmail::CreatedAt)
                .timestamp_with_time_zone()
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .to_owned()
}

fn drop_user_email_table_statement() -> TableDropStatement {
    Table::drop().table(UserEmail::Table).to_owned()
}

#[async_trait::async_trait]
impl Operation<sqlx::Sqlite> for CreateUserEmailTable {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_user_email_table_statement().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_user_email_table_statement().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}

pub struct Migration01;

sqlx_migrator::sqlite_migration!(
    Migration01,
    "main",
    "migration_01",
    vec_box![],
    vec_box![CreateUserEmailTable]
);
