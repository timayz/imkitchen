use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::User;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
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
                .string_len(320),
        )
        .col(ColumnDef::new(User::Username).string().string_len(15))
        .col(
            ColumnDef::new(User::Role)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(User::State)
                .string()
                .not_null()
                .string_len(15),
        )
        .col(
            ColumnDef::new(User::SubscriptionExpireAt)
                .big_integer()
                .not_null()
                .default(0),
        )
        .col(ColumnDef::new(User::CreatedAt).big_integer().not_null())
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(User::Table).to_owned()
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

pub struct CreateUk1;
//
// fn create_uk_1() -> IndexCreateStatement {
//     Index::create()
//         .name("uk_user_fN5xcl")
//         .table(User::Table)
//         .unique()
//         .col(User::Email)
//         .to_owned()
// }
//
// fn drop_uk_1() -> IndexDropStatement {
//     Index::drop()
//         .name("uk_user_fN5xcl")
//         .table(User::Table)
//         .to_owned()
// }

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateUk1 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        // let statement = create_uk_1().to_string(sea_query::SqliteQueryBuilder);
        // sqlx::query(&statement).execute(connection).await?;

        sqlx::query(r#"CREATE UNIQUE INDEX "uk_user_fN5xcl" on "user" ("email" COLLATE NOCASE)"#)
            .execute(connection)
            .await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        // let statement = drop_uk_1().to_string(sea_query::SqliteQueryBuilder);
        // sqlx::query(&statement).execute(connection).await?;

        sqlx::query(r#"DROP INDEX "uk_user_fN5xcl""#)
            .execute(connection)
            .await?;

        Ok(())
    }
}

pub struct CreateUk2;

// fn create_uk_2() -> IndexCreateStatement {
//     Index::create()
//         .name("uk_user_yNpV2x")
//         .table(User::Table)
//         .unique()
//         .col(Expr::cust("username COLLATE NOCASE"))
//         .to_owned()
// }
//
// fn drop_uk_2() -> IndexDropStatement {
//     Index::drop()
//         .name("uk_user_yNpV2x")
//         .table(User::Table)
//         .to_owned()
// }

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateUk2 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        // let statement = create_uk_2().to_string(sea_query::SqliteQueryBuilder);
        // sqlx::query(&statement).execute(connection).await?;

        sqlx::query(
            r#"CREATE UNIQUE INDEX "uk_user_yNpV2x" on "user" ("username" COLLATE NOCASE)"#,
        )
        .execute(connection)
        .await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        // let statement = drop_uk_2().to_string(sea_query::SqliteQueryBuilder);
        // sqlx::query(&statement).execute(connection).await?;

        sqlx::query(r#"DROP INDEX "uk_user_yNpV2x""#)
            .execute(connection)
            .await?;

        Ok(())
    }
}
