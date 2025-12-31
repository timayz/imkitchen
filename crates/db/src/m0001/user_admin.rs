use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::UserAdmin;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(UserAdmin::Table)
        .col(
            ColumnDef::new(UserAdmin::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(UserAdmin::Email)
                .string()
                .not_null()
                .string_len(320),
        )
        .col(ColumnDef::new(UserAdmin::FullName).string().string_len(25))
        .col(ColumnDef::new(UserAdmin::Username).string().string_len(15))
        .col(
            ColumnDef::new(UserAdmin::State)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(UserAdmin::Role)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(UserAdmin::TotalRecipesCount)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(UserAdmin::SharedRecipesCount)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(UserAdmin::TotalActiveCount)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(UserAdmin::SubscriptionExpireAt)
                .big_integer()
                .default(0),
        )
        .col(
            ColumnDef::new(UserAdmin::CreatedAt)
                .big_integer()
                .not_null(),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(UserAdmin::Table).to_owned()
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
        .table(UserAdmin::Table)
        .col(UserAdmin::State)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idx_user_list_D0nmGG")
        .table(UserAdmin::Table)
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
        .table(UserAdmin::Table)
        .col(UserAdmin::Role)
        .to_owned()
}

fn drop_idx_2() -> IndexDropStatement {
    Index::drop()
        .name("idx_user_list_rmnGYD")
        .table(UserAdmin::Table)
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
