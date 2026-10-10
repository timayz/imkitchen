use imkitchen_web_shared::services::tour::{Status, TourState};
use serde::{Deserialize, Serialize};

/// `status` is snake_case (`pending`, `in_progress`, `completed`,
/// `skipped`), like the kitchen `Status`.
#[derive(Serialize, Debug)]
pub struct Tour {
    pub id: &'static str,
    pub version: u16,
    /// The page/tab the tour belongs to: `kitchen`, `recipes`, `groceries`, `settings`.
    pub page: &'static str,
    pub steps: Vec<&'static str>,
    pub status: Status,
    /// Catalog index to resume at.
    pub step: u16,
}

#[derive(Serialize, Debug)]
pub struct Tours {
    pub tours: Vec<Tour>,
}

impl From<TourState> for Tours {
    fn from(state: TourState) -> Self {
        Self {
            tours: state
                .tours
                .into_iter()
                .map(|t| Tour {
                    id: t.id,
                    version: t.version,
                    page: t.page,
                    steps: t.steps.to_vec(),
                    status: t.status,
                    step: t.step,
                })
                .collect(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct StepRequest {
    pub step: u16,
}
