use evento::{Executor, ProjectionAggregate};
use imkitchen_types::tour::Completed;

use super::{TourStatus, catalog};

pub struct CompleteInput {
    pub tour: String,
}

impl<E: Executor> super::Module<E> {
    /// Marks the tour as done; a no-op when it already is.
    pub async fn complete(
        &self,
        id: impl Into<String>,
        input: CompleteInput,
    ) -> imkitchen_core::Result<()> {
        let Some(def) = catalog::find(&input.tour) else {
            imkitchen_core::not_found!("Tour");
        };

        let id = id.into();
        let tour = self.load(&id).await?;
        if tour
            .progress(def.id)
            .is_some_and(|p| p.status == TourStatus::Completed && p.version == def.version)
        {
            return Ok(());
        }

        tour.write()?
            .event(&Completed {
                tour: def.id.to_owned(),
                version: def.version,
            })
            .requested_by(id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
