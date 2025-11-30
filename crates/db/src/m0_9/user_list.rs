use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::UserList;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(UserList::Table)
        .col(
            ColumnDef::new(UserList::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(UserList::Email)
                .string()
                .not_null()
                .string_len(320),
        )
        .col(ColumnDef::new(UserList::FullName).string().string_len(25))
        .col(ColumnDef::new(UserList::Username).string().string_len(15))
        .col(
            ColumnDef::new(UserList::State)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(UserList::Role)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(UserList::TotalRecipesCount)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(UserList::SharedRecipesCount)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(UserList::TotalActiveCount)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(UserList::SubscriptionExpireAt)
                .big_integer()
                .default(0),
        )
        .col(ColumnDef::new(UserList::CreatedAt).big_integer().not_null())
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(UserList::Table).to_owned()
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
        .name("idx_user_list_D0nmGG")
        .table(UserList::Table)
        .col(UserList::State)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idx_user_list_D0nmGG")
        .table(UserList::Table)
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

pub struct CreateIdx2;

fn create_idx_2() -> IndexCreateStatement {
    Index::create()
        .name("idx_user_list_rmnGYD")
        .table(UserList::Table)
        .col(UserList::Role)
        .to_owned()
}

fn drop_idx_2() -> IndexDropStatement {
    Index::drop()
        .name("idx_user_list_rmnGYD")
        .table(UserList::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx2 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_idx_2().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_idx_2().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}
