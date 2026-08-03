use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum AudienceDailyStat {
    Table,
    Day,
    Path,
    Device,
    Browser,
    Country,
    Referrer,
    Total,
    CreatedAt,
    UpdatedAt,
}

pub(crate) mod m0001 {
    use sea_query::{
        ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
        TableDropStatement,
    };

    use super::AudienceDailyStat;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(AudienceDailyStat::Table)
            .col(
                ColumnDef::new(AudienceDailyStat::Day)
                    .string()
                    .string_len(20)
                    .not_null(),
            )
            .col(
                ColumnDef::new(AudienceDailyStat::Path)
                    .string()
                    .string_len(50)
                    .not_null(),
            )
            .col(
                ColumnDef::new(AudienceDailyStat::Device)
                    .string()
                    .string_len(50)
                    .not_null(),
            )
            .col(
                ColumnDef::new(AudienceDailyStat::Browser)
                    .string()
                    .string_len(50)
                    .not_null(),
            )
            .col(
                ColumnDef::new(AudienceDailyStat::Country)
                    .string()
                    .string_len(2)
                    .not_null(),
            )
            .col(
                ColumnDef::new(AudienceDailyStat::Referrer)
                    .string()
                    .string_len(100)
                    .not_null(),
            )
            .col(
                ColumnDef::new(AudienceDailyStat::Total)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(AudienceDailyStat::CreatedAt)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(AudienceDailyStat::UpdatedAt)
                    .big_integer()
                    .not_null(),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(AudienceDailyStat::Table).to_owned()
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

    pub struct CreateUk1;

    fn create_uk_1() -> IndexCreateStatement {
        Index::create()
            .name("uk_audience_daily_stat_dims")
            .table(AudienceDailyStat::Table)
            .unique()
            .col(AudienceDailyStat::Day)
            .col(AudienceDailyStat::Path)
            .col(AudienceDailyStat::Device)
            .col(AudienceDailyStat::Browser)
            .col(AudienceDailyStat::Country)
            .col(AudienceDailyStat::Referrer)
            .to_owned()
    }

    fn drop_uk_1() -> IndexDropStatement {
        Index::drop()
            .name("uk_audience_daily_stat_dims")
            .table(AudienceDailyStat::Table)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateUk1 {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_uk_1().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_uk_1().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }
}
