use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum ShoppingList {
    Table,
    UserId,
    Ingredients,
    FromDate,
    Days,
    GeneratedAt,
    Recipes,
}

pub(crate) mod m0001 {
    use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

    use super::ShoppingList;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(ShoppingList::Table)
            .col(
                ColumnDef::new(ShoppingList::UserId)
                    .primary_key()
                    .string()
                    .not_null()
                    .string_len(26),
            )
            .col(ColumnDef::new(ShoppingList::Ingredients).blob().not_null())
            .col(
                ColumnDef::new(ShoppingList::FromDate)
                    .big_integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(ShoppingList::Days)
                    .tiny_integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(ShoppingList::GeneratedAt)
                    .big_integer()
                    .not_null(),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(ShoppingList::Table).to_owned()
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
}

pub(crate) mod m0011 {
    use sea_query::{ColumnDef, Table, TableAlterStatement};

    use super::ShoppingList;

    pub struct AddRecipes;

    fn add_column() -> TableAlterStatement {
        // Nullable blob (bitcode-encoded Vec<String> of recipe ids). Nullable
        // avoids needing a constant SQL default for a NOT NULL blob; the read
        // layer treats NULL / pre-migration rows as an empty recipe set.
        Table::alter()
            .table(ShoppingList::Table)
            .add_column(ColumnDef::new(ShoppingList::Recipes).blob())
            .to_owned()
    }

    fn drop_column() -> TableAlterStatement {
        Table::alter()
            .table(ShoppingList::Table)
            .drop_column(ShoppingList::Recipes)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for AddRecipes {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let add_column = add_column().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(add_column))
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
