use evento::{Executor, metadata::Metadata};

use crate::Created;

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn create(
        executor: &E,
        request_by: impl Into<String>,
        owner_name: impl Into<Option<String>>,
    ) -> imkitchen_shared::Result<String> {
        Ok(evento::create()
            .event(&Created {
                name: "".to_owned(),
                owner_name: owner_name.into(),
            })
            .metadata(&Metadata::new(request_by))
            .commit(executor)
            .await?)
    }
}
