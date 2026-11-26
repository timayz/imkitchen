//! The write side of a user's tutorial: what its stream says of it, the
//! commands of the user, and the steps the api sees them do.
//!
//! The stream is under the id of the user, and starts with the first step
//! they do or with their skipping it: until then every step is ahead of
//! them.
//!
//! The state folds every event of the stream (`strict()`), so the version
//! it holds, which a write goes out at, is never stale. It is not
//! snapshotted: a tutorial is a handful of events.

use axum::http::StatusCode;
use evento::metadata::Event;
use evento::{Executor, Projection, ProjectionAggregate, WriteBuilder, WriteError};
use imkitchen_aggregates::tutorial::{Restarted, Skipped, Step, StepCompleted, Tutorial};

use crate::error::{ApiError, ApiResult};

/// How many times a write that lost to another on the same tutorial is
/// made again, over the tutorial as the other left it.
const WRITE_ATTEMPTS: u32 = 3;

/// The tutorial of a user as its stream leaves it.
#[evento::projection(id = id, name = "imkitchen/TutorialState")]
#[evento::snapshot(none)]
#[derive(Debug, PartialEq)]
pub struct TutorialState {
    /// The id of the user.
    pub id: String,
    /// The steps the user did, in the order they did them.
    pub done: Vec<Step>,
    pub skipped: bool,
    /// Unix seconds of the last `Restarted`, 0 when there is none: what
    /// the user did before does not count.
    pub restarted_at: u64,
}

impl TutorialState {
    /// Whether the user did every step.
    pub fn completed(&self) -> bool {
        Step::ALL.iter().all(|step| self.done.contains(step))
    }
}

fn projection<E: Executor>() -> Projection<E, TutorialState> {
    Projection::new::<Tutorial>()
        .handler(on_step_completed())
        .handler(on_skipped())
        .handler(on_restarted())
        .strict()
}

#[evento::handler]
async fn on_step_completed(
    event: Event<StepCompleted>,
    tutorial: &mut TutorialState,
) -> anyhow::Result<()> {
    tutorial.id = event.aggregate_id.to_owned();
    if !tutorial.done.contains(&event.data.step) {
        tutorial.done.push(event.data.step);
    }
    Ok(())
}

#[evento::handler]
async fn on_skipped(event: Event<Skipped>, tutorial: &mut TutorialState) -> anyhow::Result<()> {
    tutorial.id = event.aggregate_id.to_owned();
    tutorial.skipped = true;
    Ok(())
}

#[evento::handler]
async fn on_restarted(event: Event<Restarted>, tutorial: &mut TutorialState) -> anyhow::Result<()> {
    tutorial.done.clear();
    tutorial.skipped = false;
    tutorial.restarted_at = event.timestamp;
    Ok(())
}

/// The tutorial of `user_id` as its stream leaves it, every step ahead for
/// a user who did nothing yet.
pub async fn load<E: Executor>(executor: &E, user_id: &str) -> anyhow::Result<TutorialState> {
    let tutorial = projection().load(user_id).execute(executor).await?;
    Ok(tutorial.unwrap_or_default())
}

/// Skips the tutorial of `user_id`; one already skipped writes nothing.
pub async fn skip<E: Executor>(executor: &E, user_id: &str) -> ApiResult<()> {
    let written = write(executor, user_id, |tutorial, write| {
        if tutorial.skipped {
            return false;
        }
        write.event(&Skipped);
        true
    })
    .await?;
    written.ok_or(not_taken())
}

/// Starts the tutorial of `user_id` over; one nothing was done of writes
/// nothing.
pub async fn restart<E: Executor>(executor: &E, user_id: &str) -> ApiResult<()> {
    let written = write(executor, user_id, |tutorial, write| {
        if tutorial.done.is_empty() && !tutorial.skipped {
            return false;
        }
        write.event(&Restarted);
        true
    })
    .await?;
    written.ok_or(not_taken())
}

/// Notes that `user_id` did `step`, by a command written at `at` (unix
/// seconds). Nothing is written for a step already done, for a tutorial
/// that is skipped, or for a command from before the tutorial was started
/// over: so the same command seen twice, or by every node, counts once.
pub async fn complete<E: Executor>(
    executor: &E,
    user_id: &str,
    step: Step,
    at: u64,
) -> anyhow::Result<()> {
    let written = write(executor, user_id, |tutorial, write| {
        if tutorial.skipped || tutorial.done.contains(&step) || at < tutorial.restarted_at {
            return false;
        }
        write.event(&StepCompleted { step });
        true
    })
    .await?;
    match written {
        Some(()) => Ok(()),
        None => anyhow::bail!("the tutorial of {user_id} kept being written by others"),
    }
}

fn not_taken() -> ApiError {
    ApiError::refused(
        StatusCode::SERVICE_UNAVAILABLE,
        timada_core::WRITE_NOT_TAKEN,
    )
}

/// Appends what `change` adds to the tutorial of `user_id`, at the version
/// it was loaded at, 0 for a user who did nothing yet; `change` answers
/// `false` when there is nothing to write. A write another one got ahead
/// of is made again over the tutorial as it now stands; `None` when it
/// kept losing.
async fn write<E: Executor>(
    executor: &E,
    user_id: &str,
    change: impl Fn(&TutorialState, &mut WriteBuilder) -> bool,
) -> anyhow::Result<Option<()>> {
    for _ in 0..WRITE_ATTEMPTS {
        let tutorial = load(executor, user_id).await?;
        let mut write = evento::append(user_id);
        write.original_version(tutorial.aggregate_version()?);
        if !change(&tutorial, &mut write) {
            return Ok(Some(()));
        }
        match write.commit(executor).await {
            Ok(_) => return Ok(Some(())),
            Err(WriteError::InvalidOriginalVersion) => continue,
            Err(err) => return Err(err.into()),
        }
    }
    Ok(None)
}
