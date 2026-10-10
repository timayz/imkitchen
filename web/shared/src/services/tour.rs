//! Guided tours: the catalog (what tours exist) joined with the user's
//! progress. Clients decide which tour to open from this view.

pub use imkitchen_identity::tour::catalog::{CATALOG, TourDef};
use imkitchen_identity::tour::{AdvanceInput, CompleteInput, SkipInput};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

impl From<imkitchen_identity::tour::TourStatus> for Status {
    fn from(status: imkitchen_identity::tour::TourStatus) -> Self {
        use imkitchen_identity::tour::TourStatus as S;
        match status {
            S::Pending => Self::Pending,
            S::InProgress => Self::InProgress,
            S::Completed => Self::Completed,
            S::Skipped => Self::Skipped,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct TourView {
    pub id: &'static str,
    pub version: u16,
    pub page: &'static str,
    pub steps: &'static [&'static str],
    pub status: Status,
    /// The step to resume at (catalog index).
    pub step: u16,
}

#[derive(Serialize, Debug)]
pub struct TourState {
    pub tours: Vec<TourView>,
}

/// Every catalog tour with the user's progress. Progress recorded against
/// an older catalog version counts as pending: the tour changed, show it again.
pub async fn state(app: &AppState, user_id: &str) -> anyhow::Result<TourState> {
    let progress = app.identity.tour.load(user_id).await?;
    let tours = CATALOG
        .iter()
        .map(|def| {
            let (status, step) = match progress.progress(def.id) {
                Some(p) if p.version == def.version => (p.status.into(), p.step),
                _ => (Status::Pending, 0),
            };
            TourView {
                id: def.id,
                version: def.version,
                page: def.page,
                steps: def.steps,
                status,
                step,
            }
        })
        .collect();
    Ok(TourState { tours })
}

pub async fn advance(
    app: &AppState,
    user_id: &str,
    tour: &str,
    step: u16,
) -> imkitchen_core::Result<()> {
    app.identity
        .tour
        .advance(
            user_id,
            AdvanceInput {
                tour: tour.to_owned(),
                step,
            },
        )
        .await
}

pub async fn complete(app: &AppState, user_id: &str, tour: &str) -> imkitchen_core::Result<()> {
    app.identity
        .tour
        .complete(
            user_id,
            CompleteInput {
                tour: tour.to_owned(),
            },
        )
        .await
}

pub async fn skip(
    app: &AppState,
    user_id: &str,
    tour: &str,
    step: u16,
) -> imkitchen_core::Result<()> {
    app.identity
        .tour
        .skip(
            user_id,
            SkipInput {
                tour: tour.to_owned(),
                step,
            },
        )
        .await
}

pub async fn reset(app: &AppState, user_id: &str) -> imkitchen_core::Result<()> {
    app.identity.tour.reset(user_id).await
}
