use sea_query::{
    ColumnDef, Expr, Iden, Index, IndexCreateStatement, IndexDropStatement, Table,
    TableCreateStatement, TableDropStatement,
};
use sqlx_migrator::{Operation, vec_box};

#[derive(Iden, Clone)]
pub enum User {
    Table,
    Id,
    Email,
    CreatedAt,
}

pub struct CreateUserTable;

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
impl Operation<sqlx::Sqlite> for CreateUserTable {
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

pub struct CreateEmailIdx;

fn create_email_idx_statement() -> IndexCreateStatement {
    Index::create()
        .name("idx_email")
        .table(User::Table)
        .unique()
        .col(User::Email)
        .to_owned()
}

fn drop_email_idx_statement() -> IndexDropStatement {
    Index::drop()
        .name("idx_email")
        .table(User::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl Operation<sqlx::Sqlite> for CreateEmailIdx {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_email_idx_statement().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_email_idx_statement().to_string(sea_query::SqliteQueryBuilder);
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
    vec_box![CreateUserTable, CreateEmailIdx]
);
