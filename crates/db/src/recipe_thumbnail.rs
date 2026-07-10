use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum RecipeThumbnail {
    Table,
    Id,
    Device,
    Data,
}

pub(crate) mod m0001 {
    use sea_query::{
        ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
        TableDropStatement,
    };

    use super::RecipeThumbnail;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(RecipeThumbnail::Table)
            .col(
                ColumnDef::new(RecipeThumbnail::Id)
                    .string()
                    .not_null()
                    .string_len(26),
            )
            .col(
                ColumnDef::new(RecipeThumbnail::Device)
                    .string()
                    .not_null()
                    .string_len(10),
            )
            .col(ColumnDef::new(RecipeThumbnail::Data).blob().not_null())
            .primary_key(
                Index::create()
                    .col(RecipeThumbnail::Id)
                    .col(RecipeThumbnail::Device),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(RecipeThumbnail::Table).to_owned()
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
            .name("idx_recipe_thumbnail_6jagKS")
            .table(RecipeThumbnail::Table)
            .col(RecipeThumbnail::Id)
            .to_owned()
    }

    fn drop_idx_1() -> IndexDropStatement {
        Index::drop()
            .name("idx_recipe_thumbnail_6jagKS")
            .table(RecipeThumbnail::Table)
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

pub(crate) mod m0009 {
    /// One-time reclaim: strip the image bytes out of the thumbnail events.
    ///
    /// Historically `ThumbnailUploaded` carried the original upload bytes and
    /// `ThumbnailResized` carried the resized WebP bytes, so every image lived
    /// three times (both events plus the `recipe_thumbnail` projection). The
    /// event format is now byte-free (`ThumbnailUploaded` is a unit variant and
    /// `ThumbnailResized` only keeps `device`), and `recipe_thumbnail` is the
    /// authoritative image store.
    ///
    /// The rewrite is done in pure SQL because the new bitcode blob is a prefix
    /// of the old one (verified against every row): a unit variant encodes to an
    /// empty blob, and `ThumbnailResized { device }` encodes to just the leading
    /// length-prefixed device string. `substr` on a BLOB operates byte-wise.
    ///
    /// IRREVERSIBLE: the original upload bytes are discarded and are not stored
    /// anywhere else, so `down()` cannot restore them.
    pub struct StripThumbnailBytes;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for StripThumbnailBytes {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            // Guard: the byte-slice rewrite only handles the known device
            // prefixes 0x06 ("mobile"/"tablet") and 0x07 ("desktop"). Abort if
            // anything else exists so we never truncate to the wrong length.
            let unexpected: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM event \
                 WHERE name = 'ThumbnailResized' \
                 AND substr(data, 1, 1) NOT IN (X'06', X'07')",
            )
            .fetch_one(&mut *connection)
            .await?;

            if unexpected != 0 {
                return Err(sqlx_migrator::Error::Box(Box::from(format!(
                    "m0009: {unexpected} ThumbnailResized events have an unexpected \
                     device prefix; aborting to avoid corrupting event data"
                ))));
            }

            // ThumbnailUploaded is now a unit variant → empty blob.
            sqlx::query("UPDATE event SET data = X'' WHERE name = 'ThumbnailUploaded'")
                .execute(&mut *connection)
                .await?;

            // ThumbnailResized { device } → keep only the length-prefixed device
            // string, dropping the trailing Vec<u8>. mobile/tablet = 6-char
            // device (prefix 0x06) → keep 7 bytes; desktop = 7-char (0x07) → 8.
            sqlx::query(
                "UPDATE event \
                 SET data = CASE substr(data, 1, 1) \
                              WHEN X'06' THEN substr(data, 1, 7) \
                              WHEN X'07' THEN substr(data, 1, 8) \
                            END \
                 WHERE name = 'ThumbnailResized'",
            )
            .execute(&mut *connection)
            .await?;

            // Do NOT reset the recipe-thumbnail-view / recipe-query cursors or
            // truncate recipe_thumbnail: the variant bytes and the existing
            // blur_placeholder values are authoritative and can no longer be
            // rebuilt from events.
            Ok(())
        }

        async fn down(
            &self,
            _connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            // Irreversible: the stripped image bytes were discarded. No-op,
            // matching the precedent for irreversible data migrations.
            Ok(())
        }
    }
}
