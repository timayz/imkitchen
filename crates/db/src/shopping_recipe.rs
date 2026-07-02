use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum ShoppingRecipe {
    Table,
    Id,
    UserId,
    Ingredients,
    HouseholdSize,
}

pub(crate) mod m0001 {
    use sea_query::{
        ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
        TableDropStatement,
    };

    use super::ShoppingRecipe;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(ShoppingRecipe::Table)
            .col(
                ColumnDef::new(ShoppingRecipe::Id)
                    .string()
                    .not_null()
                    .string_len(26)
                    .primary_key(),
            )
            .col(
                ColumnDef::new(ShoppingRecipe::UserId)
                    .string()
                    .not_null()
                    .string_len(26),
            )
            .col(
                ColumnDef::new(ShoppingRecipe::Ingredients)
                    .blob()
                    .not_null(),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(ShoppingRecipe::Table).to_owned()
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
            .name("idx_shopping_recipe_KSDt5k")
            .table(ShoppingRecipe::Table)
            .col(ShoppingRecipe::UserId)
            .to_owned()
    }

    fn drop_idx_1() -> IndexDropStatement {
        Index::drop()
            .name("idx_shopping_recipe_KSDt5k")
            .table(ShoppingRecipe::Table)
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

pub(crate) mod m0007 {
    use sea_query::{ColumnDef, Table, TableAlterStatement};

    use super::ShoppingRecipe;

    pub struct AddHouseholdSize;

    fn add_column() -> TableAlterStatement {
        Table::alter()
            .table(ShoppingRecipe::Table)
            .add_column(
                ColumnDef::new(ShoppingRecipe::HouseholdSize)
                    .integer()
                    .not_null()
                    .default(4),
            )
            .to_owned()
    }

    fn drop_column() -> TableAlterStatement {
        Table::alter()
            .table(ShoppingRecipe::Table)
            .drop_column(ShoppingRecipe::HouseholdSize)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for AddHouseholdSize {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let add_column = add_column().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(add_column))
                .execute(&mut *connection)
                .await?;

            // Backfill from the recipe_user projection, which already carries the
            // authored household size for every recipe. New rows default to 4.
            let backfill = "UPDATE shopping_recipe SET household_size = COALESCE((SELECT household_size FROM recipe_user WHERE recipe_user.id = shopping_recipe.id), 4)";
            sqlx::query(sqlx::AssertSqlSafe(backfill))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let drop_column = drop_column().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(drop_column))
                .execute(connection)
                .await?;

            Ok(())
        }
    }
}
