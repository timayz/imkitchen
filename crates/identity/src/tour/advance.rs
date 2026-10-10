use evento::{Executor, ProjectionAggregate};
use imkitchen_types::tour::Advanced;

use super::{TourStatus, catalog};

pub struct AdvanceInput {
    pub tour: String,
    pub step: u16,
}

impl<E: Executor> super::Module<E> {
    /// Records that the user is on `step`; a no-op when already there.
    pub async fn advance(
        &self,
        id: impl Into<String>,
        input: AdvanceInput,
    ) -> imkitchen_core::Result<()> {
        let Some(def) = catalog::find(&input.tour) else {
            imkitchen_core::not_found!("Tour");
        };
        if usize::from(input.step) >= def.steps.len() {
            imkitchen_core::user!("Step {} is out of range", input.step);
        }

        let id = id.into();
        let tour = self.load(&id).await?;
        if tour.progress(def.id).is_some_and(|p| {
            p.status == TourStatus::InProgress && p.step == input.step && p.version == def.version
        }) {
            return Ok(());
        }

        tour.write()?
            .event(&Advanced {
                tour: def.id.to_owned(),
                step: input.step,
                version: def.version,
            })
            .requested_by(id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
