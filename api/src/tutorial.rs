//! `/v1/tutorial/*`: how far a user is through what the app walks them
//! through.
//!
//! A user has one tutorial, their own: every route takes [`Session`] and
//! acts on the tutorial of the user it signs in. It is there from the
//! start, every step ahead, and kept here so that it is the same on every
//! device.
//!
//! No route does a step: each is done by the command it names, wherever it
//! is made from, and noted a moment later ([`subscription`]). The user
//! skips the tutorial or starts it over. It is read from its stream, so it
//! is as the last command, or the last step noted, left it.

mod model;
mod progress;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use imkitchen_aggregates::tutorial::Step;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub use self::progress::{SUBSCRIPTION, subscription};
use crate::auth::Session;
use crate::error::ApiResult;
use crate::openapi::TUTORIAL;
use crate::server::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(tutorial))
        .routes(routes!(skip))
        .routes(routes!(restart))
}

/// What a step of the tutorial asks of the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum StepName {
    /// Write a recipe, or import one.
    Recipe,
    /// Compose a plat.
    Plat,
    /// Set how the list is generated, or the language.
    Settings,
    /// Generate the list, or pick a meal.
    List,
    /// Check an item of the groceries, or adjust one.
    Groceries,
}

impl From<Step> for StepName {
    fn from(step: Step) -> Self {
        match step {
            Step::Recipe => Self::Recipe,
            Step::Plat => Self::Plat,
            Step::Settings => Self::Settings,
            Step::List => Self::List,
            Step::Groceries => Self::Groceries,
        }
    }
}

/// One step of the tutorial.
#[derive(Serialize, ToSchema)]
struct StepProgress {
    step: StepName,
    /// The user did it.
    completed: bool,
}

/// The tutorial of a user.
#[derive(Serialize, ToSchema)]
struct Tutorial {
    /// Every step, in the order the tutorial goes through them.
    steps: Vec<StepProgress>,
    /// The user does not want to be walked through.
    skipped: bool,
    /// Every step is done.
    completed: bool,
}

/// The tutorial of the user
///
/// Every step and whether they did it. A step is done by the command it
/// names and shows here a moment later; one done before the tutorial was
/// skipped stays done.
#[utoipa::path(
    get,
    path = "/tutorial",
    tag = TUTORIAL,
    security(("session" = [])),
    responses((status = OK, body = Tutorial))
)]
async fn tutorial(State(state): State<AppState>, session: Session) -> ApiResult<Json<Tutorial>> {
    let tutorial = model::load(&state.executor, &session.user.id).await?;
    let steps = Step::ALL
        .into_iter()
        .map(|step| StepProgress {
            step: step.into(),
            completed: tutorial.done.contains(&step),
        })
        .collect();
    Ok(Json(Tutorial {
        steps,
        skipped: tutorial.skipped,
        completed: tutorial.completed(),
    }))
}

/// Skips the tutorial of the user
///
/// On every device. The steps they did stay done, and no other is noted
/// until they start it over.
#[utoipa::path(
    post,
    path = "/tutorial/skip",
    tag = TUTORIAL,
    security(("session" = [])),
    responses((status = NO_CONTENT, description = "The tutorial is skipped"))
)]
async fn skip(State(state): State<AppState>, session: Session) -> ApiResult<StatusCode> {
    model::skip(&state.executor, &session.user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Starts the tutorial of the user over
///
/// No step is done and it is not skipped any more: each step is done by
/// the next command that does it, whatever the user already has.
#[utoipa::path(
    post,
    path = "/tutorial/restart",
    tag = TUTORIAL,
    security(("session" = [])),
    responses((status = NO_CONTENT, description = "The tutorial is started over"))
)]
async fn restart(State(state): State<AppState>, session: Session) -> ApiResult<StatusCode> {
    model::restart(&state.executor, &session.user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
