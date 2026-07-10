use evento::{Executor, ProjectionAggregate};
use imkitchen_db::recipe_thumbnail::RecipeThumbnail;
use imkitchen_types::recipe::ThumbnailUploaded;
use sea_query::{OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

impl<E: Executor + Clone> super::Module<E> {
    pub async fn upload_thumbnail(
        &self,
        id: impl Into<String>,
        data: Vec<u8>,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let Some(recipe) = self.load(id).await? else {
            crate::not_found!("recipe");
        };

        let request_by = request_by.into();
        if recipe.owner_id != request_by {
            crate::forbidden!("not owner of recipe");
        }

        if let Err(err) = image::load_from_memory(&data) {
            crate::user!("{err}");
        };

        // Stash the original transiently in recipe_thumbnail (device='original')
        // so the async resize subscription can read it without the bytes ever
        // entering the event log. The row is deleted by the resize handler once
        // the variants are produced. Written before the event is committed so
        // the subscription is guaranteed to see it.
        let statement = Query::insert()
            .into_table(RecipeThumbnail::Table)
            .columns([
                RecipeThumbnail::Id,
                RecipeThumbnail::Device,
                RecipeThumbnail::Data,
            ])
            .values_panic([recipe.id.to_owned().into(), "original".into(), data.into()])
            .on_conflict(
                OnConflict::columns([RecipeThumbnail::Id, RecipeThumbnail::Device])
                    .update_column(RecipeThumbnail::Data)
                    .to_owned(),
            )
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
            .execute(&self.write_db)
            .await?;

        recipe
            .write()?
            .event(&ThumbnailUploaded)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
