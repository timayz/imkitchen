use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum RecipeUser {
    Table,
    Id,
    Cursor,
    OwnerId,
    OwnerName,
    RecipeType,
    CuisineType,
    Name,
    Slug,
    Origin,
    Description,
    HouseholdSize,
    PrepTime,
    CookTime,
    Ingredients,
    Instructions,
    DietaryRestrictions,
    AcceptsAccompaniment,
    AdvancePrep,
    IsShared,
    CreatedAt,
    UpdatedAt,
    ThumbnailVersion,
    DifficultyScore,
    BlurPlaceholder,
}

#[derive(Iden, Clone)]
pub enum RecipeUserFts {
    Table,
    Id,
    Name,
    Description,
    Ingredients,
    Rank,
}

pub(crate) mod m0001 {
    use sea_query::{
        ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
        TableDropStatement,
    };

    use super::RecipeUser;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(RecipeUser::Table)
            .col(
                ColumnDef::new(RecipeUser::Id)
                    .string()
                    .not_null()
                    .string_len(26)
                    .primary_key(),
            )
            .col(ColumnDef::new(RecipeUser::Cursor).string().not_null())
            .col(
                ColumnDef::new(RecipeUser::OwnerId)
                    .string()
                    .not_null()
                    .string_len(26),
            )
            .col(
                ColumnDef::new(RecipeUser::OwnerName)
                    .string()
                    .string_len(15),
            )
            .col(
                ColumnDef::new(RecipeUser::RecipeType)
                    .string()
                    .not_null()
                    .string_len(25),
            )
            .col(
                ColumnDef::new(RecipeUser::CuisineType)
                    .string()
                    .not_null()
                    .string_len(25),
            )
            .col(
                ColumnDef::new(RecipeUser::Name)
                    .string()
                    .not_null()
                    .string_len(100),
            )
            .col(ColumnDef::new(RecipeUser::Origin).string().string_len(255))
            .col(
                ColumnDef::new(RecipeUser::Description)
                    .string()
                    .not_null()
                    .string_len(2000)
                    .default(""),
            )
            .col(
                ColumnDef::new(RecipeUser::HouseholdSize)
                    .integer()
                    .not_null()
                    .default(4),
            )
            .col(
                ColumnDef::new(RecipeUser::PrepTime)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(RecipeUser::CookTime)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(ColumnDef::new(RecipeUser::Ingredients).blob().not_null())
            .col(ColumnDef::new(RecipeUser::Instructions).blob().not_null())
            .col(
                ColumnDef::new(RecipeUser::DietaryRestrictions)
                    .json_binary()
                    .not_null(),
            )
            .col(
                ColumnDef::new(RecipeUser::AcceptsAccompaniment)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(RecipeUser::AdvancePrep)
                    .string()
                    .not_null()
                    .string_len(2000)
                    .default(""),
            )
            .col(
                ColumnDef::new(RecipeUser::IsShared)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(RecipeUser::ThumbnailVersion)
                    .string()
                    .string_len(26),
            )
            .col(
                ColumnDef::new(RecipeUser::CreatedAt)
                    .big_integer()
                    .not_null(),
            )
            .col(ColumnDef::new(RecipeUser::UpdatedAt).big_integer().null())
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(RecipeUser::Table).to_owned()
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
            .name("idx_recipe_user_6jagKS")
            .table(RecipeUser::Table)
            .col(RecipeUser::OwnerId)
            .col(RecipeUser::CuisineType)
            .to_owned()
    }

    fn drop_idx_1() -> IndexDropStatement {
        Index::drop()
            .name("idx_recipe_user_6jagKS")
            .table(RecipeUser::Table)
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

    pub struct CreateIdx2;

    fn create_idx_2() -> IndexCreateStatement {
        Index::create()
            .name("idx_recipe_user_FctFNN")
            .table(RecipeUser::Table)
            .col(RecipeUser::OwnerId)
            .col(RecipeUser::RecipeType)
            .to_owned()
    }

    fn drop_idx_2() -> IndexDropStatement {
        Index::drop()
            .name("idx_recipe_user_FctFNN")
            .table(RecipeUser::Table)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx2 {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_idx_2().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_idx_2().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    pub struct CreateIdx3;

    fn create_idx_3() -> IndexCreateStatement {
        Index::create()
            .name("idx_recipe_user_KSDt5k")
            .table(RecipeUser::Table)
            .col(RecipeUser::OwnerId)
            .to_owned()
    }

    fn drop_idx_3() -> IndexDropStatement {
        Index::drop()
            .name("idx_recipe_user_KSDt5k")
            .table(RecipeUser::Table)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx3 {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_idx_3().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_idx_3().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    pub struct CreateIdx4;

    fn create_idx_4() -> IndexCreateStatement {
        Index::create()
            .name("idx_recipe_user_QJBhvl")
            .table(RecipeUser::Table)
            .col(RecipeUser::IsShared)
            .col(RecipeUser::CuisineType)
            .to_owned()
    }

    fn drop_idx_4() -> IndexDropStatement {
        Index::drop()
            .name("idx_recipe_user_QJBhvl")
            .table(RecipeUser::Table)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx4 {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_idx_4().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_idx_4().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    pub struct CreateIdx5;

    fn create_idx_5() -> IndexCreateStatement {
        Index::create()
            .name("idx_recipe_user_kXJfAR")
            .table(RecipeUser::Table)
            .col(RecipeUser::IsShared)
            .col(RecipeUser::RecipeType)
            .to_owned()
    }

    fn drop_idx_5() -> IndexDropStatement {
        Index::drop()
            .name("idx_recipe_user_kXJfAR")
            .table(RecipeUser::Table)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx5 {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_idx_5().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_idx_5().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    pub struct CreateIdx6;

    fn create_idx_6() -> IndexCreateStatement {
        Index::create()
            .name("idx_recipe_user_P4CTqO")
            .table(RecipeUser::Table)
            .col(RecipeUser::IsShared)
            .to_owned()
    }

    fn drop_idx_6() -> IndexDropStatement {
        Index::drop()
            .name("idx_recipe_user_P4CTqO")
            .table(RecipeUser::Table)
            .to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx6 {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_idx_6().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_idx_6().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    pub struct CreateFTSTable;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateFTSTable {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query(
                r#"
CREATE VIRTUAL TABLE recipe_user_fts USING fts5(id, name, description, ingredients);
            "#,
            )
            .execute(connection)
            .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query(
                r#"
DROP TABLE recipe_user_fts;
            "#,
            )
            .execute(connection)
            .await?;

            Ok(())
        }
    }
}

pub(crate) mod m0002 {
    pub struct AddDifficultyScore;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for AddDifficultyScore {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query(
                "ALTER TABLE recipe_user ADD COLUMN difficulty_score INTEGER NOT NULL DEFAULT 0",
            )
            .execute(&mut *connection)
            .await?;

            sqlx::query("UPDATE recipe_user SET difficulty_score = prep_time + cook_time")
                .execute(&mut *connection)
                .await?;

            sqlx::query("UPDATE subscriber SET cursor = NULL WHERE key = 'recipe-query'")
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query("ALTER TABLE recipe_user DROP COLUMN difficulty_score")
                .execute(connection)
                .await?;

            Ok(())
        }
    }
}

pub(crate) mod m0003 {
    pub struct DropRatingColumns;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for DropRatingColumns {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query("ALTER TABLE recipe_user DROP COLUMN total_views")
                .execute(&mut *connection)
                .await
                .ok();
            sqlx::query("ALTER TABLE recipe_user DROP COLUMN total_likes")
                .execute(&mut *connection)
                .await
                .ok();
            sqlx::query("ALTER TABLE recipe_user DROP COLUMN total_comments")
                .execute(&mut *connection)
                .await
                .ok();
            sqlx::query("DROP TABLE IF EXISTS recipe_comment")
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            _connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            Ok(())
        }
    }
}

pub(crate) mod m0004 {
    pub struct DropCuisineType;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for DropCuisineType {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query("DROP INDEX IF EXISTS idx_recipe_user_6jagKS")
                .execute(&mut *connection)
                .await?;
            sqlx::query("DROP INDEX IF EXISTS idx_recipe_user_QJBhvl")
                .execute(&mut *connection)
                .await?;
            sqlx::query("ALTER TABLE recipe_user DROP COLUMN cuisine_type")
                .execute(&mut *connection)
                .await
                .ok();
            sqlx::query("UPDATE subscriber SET cursor = NULL WHERE key = 'recipe-query'")
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            _connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            Ok(())
        }
    }
}

pub(crate) mod m0005 {
    pub struct AddSlug;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for AddSlug {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query("ALTER TABLE recipe_user ADD COLUMN slug TEXT NOT NULL DEFAULT ''")
                .execute(&mut *connection)
                .await
                .ok();

            // Slugs are derived during projection, so drop every row and replay
            // the recipe-query subscription from the start to backfill them.
            // `is_shared` rebuilds correctly from each recipe's own
            // SharedToCommunity/MadePrivate events (the recipe-saga-share saga
            // emits those per recipe), so only this subscription is reset.
            //
            // The truncate must precede the UNIQUE index: every pre-existing row
            // would otherwise share the empty default slug and collide.
            sqlx::query("DELETE FROM recipe_user")
                .execute(&mut *connection)
                .await?;

            sqlx::query(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_recipe_user_slug ON recipe_user (slug)",
            )
            .execute(&mut *connection)
            .await?;

            sqlx::query("UPDATE subscriber SET cursor = NULL WHERE key = 'recipe-query'")
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query("DROP INDEX IF EXISTS idx_recipe_user_slug")
                .execute(&mut *connection)
                .await?;
            sqlx::query("ALTER TABLE recipe_user DROP COLUMN slug")
                .execute(connection)
                .await
                .ok();

            Ok(())
        }
    }
}

pub(crate) mod m0010 {
    use sea_query::{DeleteStatement, Query};

    use super::RecipeUserFts;

    pub struct RebuildFts;

    // SQLite has no TRUNCATE — an unqualified DELETE triggers SQLite's truncate
    // optimization (sqlite.org/lang_delete.html). On other backends sea-query
    // will render the equivalent DELETE.
    fn truncate_fts() -> DeleteStatement {
        Query::delete().from_table(RecipeUserFts::Table).to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for RebuildFts {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            // `handle_ingredients_changed` previously wrote the ingredient list
            // into the FTS `name` column instead of `ingredients`, corrupting the
            // index. Truncate the projection and reset its subscription cursor so
            // the corrected handlers replay every event and rebuild it.
            let truncate = truncate_fts().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(truncate))
                .execute(&mut *connection)
                .await?;

            // `subscriber` is evento's internal table with no `Iden`, so the reset
            // is raw SQL — the same way m0002/m0004/m0005/m0007 touch it.
            sqlx::query("UPDATE subscriber SET cursor = NULL WHERE key = 'recipe-user-fts-query'")
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            _connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            Ok(())
        }
    }
}

pub(crate) mod m0007 {
    pub struct AddBlurPlaceholder;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for AddBlurPlaceholder {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query("ALTER TABLE recipe_user ADD COLUMN blur_placeholder TEXT")
                .execute(&mut *connection)
                .await
                .ok();

            // The blur placeholder is derived during projection from the mobile
            // thumbnail variant, so reset the subscription to replay every
            // ThumbnailResized event and backfill existing recipes. The column is
            // nullable, so no truncate is required.
            sqlx::query("UPDATE subscriber SET cursor = NULL WHERE key = 'recipe-query'")
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query("ALTER TABLE recipe_user DROP COLUMN blur_placeholder")
                .execute(connection)
                .await
                .ok();

            Ok(())
        }
    }
}
