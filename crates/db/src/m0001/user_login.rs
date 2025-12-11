use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

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
        .col(
            ColumnDef::new(UserLogin::UserId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(UserLogin::Revision)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(ColumnDef::new(UserLogin::UserAgent).string().not_null())
        .col(
            ColumnDef::new(UserLogin::CreatedAt)
                .big_integer()
                .not_null(),
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

pub struct CreateIdx1;

fn create_idx_1() -> IndexCreateStatement {
    Index::create()
        .name("idx_login_fN5xcl")
        .table(UserLogin::Table)
        .col(UserLogin::UserId)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idx_login_fN5xcl")
        .table(UserLogin::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx1 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_idx_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_idx_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}

pub struct CreateUk1;

fn create_uk_1() -> IndexCreateStatement {
    Index::create()
        .name("uk_login_fN5xcl")
        .table(UserLogin::Table)
        .unique()
        .col(UserLogin::UserId)
        .col(UserLogin::UserAgent)
        .to_owned()
}

fn drop_uk_1() -> IndexDropStatement {
    Index::drop()
        .name("uk_login_fN5xcl")
        .table(UserLogin::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateUk1 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_uk_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_uk_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}
