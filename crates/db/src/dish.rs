use sea_query::Iden;

/// Read model of the user's dishes (`imkitchen-core/dish`): one row per
/// live dish, with its recipe ids in meal order as a JSON array.
#[derive(Iden, Clone)]
pub enum Dish {
    Table,
    Id,
    UserId,
    Name,
    RecipeIds,
    CreatedAt,
    UpdatedAt,
}

pub(crate) mod m0016 {
    use sea_query::{
        ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
        TableDropStatement,
    };

    use super::Dish;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(Dish::Table)
            .col(
                ColumnDef::new(Dish::Id)
                    .string()
                    .not_null()
                    .string_len(26)
                    .primary_key(),
            )
            .col(
                ColumnDef::new(Dish::UserId)
                    .string()
                    .not_null()
                    .string_len(26),
            )
            .col(
                ColumnDef::new(Dish::Name)
                    .string()
                    .not_null()
                    .string_len(100),
            )
            .col(ColumnDef::new(Dish::RecipeIds).json_binary().not_null())
            .col(ColumnDef::new(Dish::CreatedAt).integer().not_null())
            .col(ColumnDef::new(Dish::UpdatedAt).integer().not_null())
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(Dish::Table).to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateTable {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_table().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_table().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    pub struct CreateIdx1;

    fn create_idx_1() -> IndexCreateStatement {
        Index::create()
            .name("idx_dish_user_id")
            .table(Dish::Table)
            .col(Dish::UserId)
            .to_owned()
    }

    fn drop_idx_1() -> IndexDropStatement {
        Index::drop()
            .name("idx_dish_user_id")
            .table(Dish::Table)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx1 {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_idx_1().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_idx_1().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }
}
