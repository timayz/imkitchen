use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::UserAuth;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(UserAuth::Table)
        .col(
            ColumnDef::new(UserAuth::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(UserAuth::Email)
                .string()
                .not_null()
                .string_len(320),
        )
        .col(
            ColumnDef::new(UserAuth::Role)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(UserAuth::State)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(UserAuth::SubscriptionExpireAt)
                .big_integer()
                .not_null()
                .default(0),
        )
        .col(ColumnDef::new(UserAuth::CreatedAt).big_integer().not_null())
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(UserAuth::Table).to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateTable {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_table().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_table().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}

pub struct CreateUk1;

fn create_uk_1() -> IndexCreateStatement {
    Index::create()
        .name("uk_user_auth_fN5xcl")
        .table(UserAuth::Table)
        .unique()
        .col(UserAuth::Email)
        .to_owned()
}

fn drop_uk_1() -> IndexDropStatement {
    Index::drop()
        .name("uk_user_auth_fN5xcl")
        .table(UserAuth::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateUk1 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_uk_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_uk_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}
