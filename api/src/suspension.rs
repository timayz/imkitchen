//! The users an administrator keeps out of imkitchen: suspended, a user's
//! account and all that is theirs stay as they are, but no session of
//! theirs signs them in and none is opened, until they are reactivated.
//!
//! Identity (`timada-identity`) knows nothing of it: a suspension is a
//! stream of the api's own, under the id of the user. Suspending ends
//! every session of the user, and [`crate::auth::sign_in`] refuses to open
//! another.
//!
//! The state folds every event of the stream (`strict()`), so the version
//! it holds, which a write goes out at, is never stale. It is not
//! snapshotted: a user is suspended a handful of times at most.

use axum::http::StatusCode;
use evento::metadata::Event;
use evento::{Executor, Projection, ProjectionAggregate, WriteBuilder, WriteError};
use imkitchen_aggregates::suspension::{Reactivated, Suspended, Suspension};

use crate::error::{ApiError, ApiResult};

/// The account is suspended: nothing signs it in until an administrator
/// reactivates it.
pub const ACCOUNT_SUSPENDED: &str = "account_suspended";

/// How many times a write that lost to another on the same suspension is
/// made again, over the suspension as the other left it.
const WRITE_ATTEMPTS: u32 = 3;

/// Whether a user is suspended, as their stream leaves it.
#[evento::projection(id = id, name = "imkitchen/SuspensionState")]
#[evento::snapshot(none)]
#[derive(Debug, PartialEq)]
pub struct SuspensionState {
    /// The id of the user.
    pub id: String,
    pub suspended: bool,
}

fn projection<E: Executor>() -> Projection<E, SuspensionState> {
    Projection::new::<Suspension>()
        .handler(on_suspended())
        .handler(on_reactivated())
        .strict()
}

#[evento::handler]
async fn on_suspended(
    event: Event<Suspended>,
    suspension: &mut SuspensionState,
) -> anyhow::Result<()> {
    suspension.id = event.aggregate_id.to_owned();
    suspension.suspended = true;
    Ok(())
}

#[evento::handler]
async fn on_reactivated(
    event: Event<Reactivated>,
    suspension: &mut SuspensionState,
) -> anyhow::Result<()> {
    suspension.id = event.aggregate_id.to_owned();
    suspension.suspended = false;
    Ok(())
}

/// Whether `user_id` is suspended.
pub async fn is_suspended<E: Executor>(executor: &E, user_id: &str) -> anyhow::Result<bool> {
    let suspension = projection().load(user_id).execute(executor).await?;
    Ok(suspension.is_some_and(|suspension| suspension.suspended))
}

/// Refuses `user_id` when they are suspended.
pub async fn refuse_suspended<E: Executor>(executor: &E, user_id: &str) -> ApiResult<()> {
    if is_suspended(executor, user_id).await? {
        return Err(ApiError::refused(StatusCode::FORBIDDEN, ACCOUNT_SUSPENDED));
    }
    Ok(())
}

/// Suspends `user_id`, as the administrator `by` asks; one who is
/// suspended already writes nothing.
pub async fn suspend<E: Executor>(executor: &E, user_id: &str, by: &str) -> ApiResult<()> {
    write(executor, user_id, |suspension, write| {
        if suspension.suspended {
            return false;
        }
        write.event(&Suspended { by: by.to_owned() });
        true
    })
    .await
}

/// Lets `user_id` in again, as the administrator `by` asks; one who is not
/// suspended writes nothing.
pub async fn reactivate<E: Executor>(executor: &E, user_id: &str, by: &str) -> ApiResult<()> {
    write(executor, user_id, |suspension, write| {
        if !suspension.suspended {
            return false;
        }
        write.event(&Reactivated { by: by.to_owned() });
        true
    })
    .await
}

/// Appends what `change` adds to the suspension of `user_id`, at the
/// version it was loaded at, 0 for a user who never was suspended; `change`
/// answers `false` when there is nothing to write. A write another one got
/// ahead of is made again over the suspension as it now stands.
async fn write<E: Executor>(
    executor: &E,
    user_id: &str,
    change: impl Fn(&SuspensionState, &mut WriteBuilder) -> bool,
) -> ApiResult<()> {
    for _ in 0..WRITE_ATTEMPTS {
        let suspension = projection()
            .load(user_id)
            .execute(executor)
            .await?
            .unwrap_or_default();
        let mut write = evento::append(user_id);
        write.original_version(suspension.aggregate_version()?);
        if !change(&suspension, &mut write) {
            return Ok(());
        }
        match write.commit(executor).await {
            Ok(_) => return Ok(()),
            Err(WriteError::InvalidOriginalVersion) => continue,
            Err(err) => return Err(err.into()),
        }
    }
    Err(ApiError::refused(
        StatusCode::SERVICE_UNAVAILABLE,
        timada_core::WRITE_NOT_TAKEN,
    ))
}
