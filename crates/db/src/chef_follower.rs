//! `chef_follower` — one row per (follower, chef) pair, maintained by the
//! recipe-follower-view subscription. `chef_name` is the username captured
//! when the follow happened and builds the `/cooks/{username}` link.

use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum ChefFollower {
    Table,
    UserId,
    ChefId,
    ChefName,
    CreatedAt,
}

pub(crate) mod m0019 {
    use sea_query::{ColumnDef, Index, SqliteQueryBuilder, Table};

    use super::ChefFollower;

    pub struct CreateTable;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateTable {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let table = Table::create()
                .table(ChefFollower::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ChefFollower::UserId)
                        .string()
                        .not_null()
                        .string_len(26),
                )
                .col(
                    ColumnDef::new(ChefFollower::ChefId)
                        .string()
                        .not_null()
                        .string_len(26),
                )
                .col(ColumnDef::new(ChefFollower::ChefName).string().not_null())
                .col(ColumnDef::new(ChefFollower::CreatedAt).integer().not_null())
                .primary_key(
                    Index::create()
                        .col(ChefFollower::UserId)
                        .col(ChefFollower::ChefId),
                )
                .to_owned();

            sqlx::query(sqlx::AssertSqlSafe(table.to_string(SqliteQueryBuilder)))
                .execute(&mut *connection)
                .await?;

            let index = Index::create()
                .if_not_exists()
                .name("idx_chef_follower_chef")
                .table(ChefFollower::Table)
                .col(ChefFollower::ChefId)
                .to_owned();

            sqlx::query(sqlx::AssertSqlSafe(index.to_string(SqliteQueryBuilder)))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let table = Table::drop()
                .table(ChefFollower::Table)
                .if_exists()
                .to_owned();

            sqlx::query(sqlx::AssertSqlSafe(table.to_string(SqliteQueryBuilder)))
                .execute(connection)
                .await?;

            Ok(())
        }
    }
}
