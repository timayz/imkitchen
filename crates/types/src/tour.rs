//! Guided tour progress, one aggregate per user (aggregate id = user id).
//!
//! `tour` is the catalog id as a string (not an enum) so adding a tour never
//! changes a persisted shape. `step` is the index into the catalog's step
//! list and `version` the catalog version the client was shown.

#[evento::aggregate]
pub enum Tour {
    /// The user reached `step` of `tour`.
    Advanced {
        tour: String,
        step: u16,
        version: u16,
    },
    /// The user went through the last step.
    Completed { tour: String, version: u16 },
    /// The user dismissed the tour at `step`.
    Skipped {
        tour: String,
        step: u16,
        version: u16,
    },
    /// Every tour pending again (Settings → "Replay the tours").
    Reset,
}
