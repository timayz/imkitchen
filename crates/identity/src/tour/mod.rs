//! Guided tour progress per user: a bitcode snapshot projection (no SQL
//! table) rebuilt from the `Tour` events, like `UserProfile`.

mod advance;
pub mod catalog;
mod complete;
mod reset;
mod skip;

use std::ops::Deref;

pub use advance::*;
pub use complete::*;
pub use skip::*;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, metadata::Event};
use imkitchen_types::tour::{self, Advanced, Completed, Reset, Skipped};

#[derive(Clone)]
pub struct Module<E: Executor>(pub(crate) imkitchen_core::State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = imkitchen_core::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Tour> {
        let id = id.into();

        create_projection::<E>()
            .load(&id)
            .execute(&self.executor)
            .await
            .map(|r| {
                r.unwrap_or_else(|| Tour {
                    id,
                    tours: Vec::new(),
                    cursor: Default::default(),
                    aggregate_version: Default::default(),
                })
            })
    }
}

#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TourStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TourProgress {
    pub tour: String,
    pub status: TourStatus,
    pub step: u16,
    /// The catalog version the user saw; older than the catalog = pending again.
    pub version: u16,
}

#[evento::projection(
    name = "imkitchen-identity/tour/Tour",
    bitcode::Encode,
    bitcode::Decode
)]
pub struct Tour {
    pub id: String,
    pub tours: Vec<TourProgress>,
}

impl Tour {
    pub fn progress(&self, tour: &str) -> Option<&TourProgress> {
        self.tours.iter().find(|p| p.tour == tour)
    }

    fn upsert(&mut self, tour: String, status: TourStatus, step: u16, version: u16) {
        match self.tours.iter_mut().find(|p| p.tour == tour) {
            Some(progress) => {
                progress.status = status;
                progress.step = step;
                progress.version = version;
            }
            None => self.tours.push(TourProgress {
                tour,
                status,
                step,
                version,
            }),
        }
    }
}

fn create_projection<E: Executor>() -> Projection<E, Tour> {
    Projection::new::<tour::Tour>()
        .handler(handle_advanced())
        .handler(handle_completed())
        .handler(handle_skipped())
        .handler(handle_reset())
        .strict()
        .revision(1)
}

impl evento::ProjectionAggregate for Tour {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_advanced(event: Event<Advanced>, data: &mut Tour) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.upsert(
        event.data.tour,
        TourStatus::InProgress,
        event.data.step,
        event.data.version,
    );
    Ok(())
}

#[evento::handler]
async fn handle_completed(event: Event<Completed>, data: &mut Tour) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    let step = data.progress(&event.data.tour).map(|p| p.step).unwrap_or(0);
    data.upsert(
        event.data.tour,
        TourStatus::Completed,
        step,
        event.data.version,
    );
    Ok(())
}

#[evento::handler]
async fn handle_skipped(event: Event<Skipped>, data: &mut Tour) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.upsert(
        event.data.tour,
        TourStatus::Skipped,
        event.data.step,
        event.data.version,
    );
    Ok(())
}

#[evento::handler]
async fn handle_reset(event: Event<Reset>, data: &mut Tour) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.tours.clear();
    Ok(())
}
