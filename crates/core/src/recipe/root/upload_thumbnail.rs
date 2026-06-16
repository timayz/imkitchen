use evento::{Executor, ProjectionAggregate};
use imkitchen_types::recipe::ThumbnailUploaded;

impl<E: Executor + Clone> super::Module<E> {
    pub async fn upload_thunmnail(
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

        recipe
            .write()?
            .event(&ThumbnailUploaded { data })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
