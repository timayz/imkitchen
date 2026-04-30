use evento::Executor;
use imkitchen_types::recipe::Created;

impl<E: Executor> super::Module<E> {
    pub async fn create(
        &self,
        request_by: impl Into<String>,
        owner_name: impl Into<Option<String>>,
    ) -> crate::Result<String> {
        Ok(evento::create()
            .event(&Created {
                name: "".to_owned(),
                owner_name: owner_name.into(),
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?)
    }
}
