//! The write side of a user's settings: what their stream says of them, and
//! the commands of the user.
//!
//! The stream is under the id of the user, and starts with the first
//! setting they change: until then, and for whatever it says nothing of,
//! the settings are the defaults ([`ListSettings::default`], no language).
//!
//! The state folds every event of the stream (`strict()`), so the version
//! it holds, which a write goes out at, is never stale. It is not
//! snapshotted: settings are changed a few dozen times at most.

use axum::http::StatusCode;
use evento::metadata::Event;
use evento::{Executor, Projection, ProjectionAggregate, WriteBuilder, WriteError};
use imkitchen_aggregates::settings::{
    DEFAULT_MEALS, DEFAULT_SERVINGS, LanguageChanged, ListChanged, ListSources, Settings,
};

use crate::error::{ApiError, ApiResult};
use crate::locale::Locale;

/// How many times a write that lost to another on the same settings is made
/// again, over the settings as the other left them.
const WRITE_ATTEMPTS: u32 = 3;

/// How a user's list is generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListSettings {
    /// How many people the list is cooked for.
    pub servings: u32,
    /// How many meals a generated list holds.
    pub meals: u32,
    /// What its meals are drawn from.
    pub sources: ListSources,
}

impl Default for ListSettings {
    /// What generates the list of a user who said nothing: a week of
    /// dinners for four, of everything they have.
    fn default() -> Self {
        Self {
            servings: DEFAULT_SERVINGS,
            meals: DEFAULT_MEALS,
            sources: ListSources::default(),
        }
    }
}

/// The settings of a user as their stream leaves them.
#[evento::projection(id = id, name = "imkitchen/SettingsState")]
#[evento::snapshot(none)]
#[derive(Debug, PartialEq)]
pub struct SettingsState {
    /// The id of the user.
    pub id: String,
    pub list: ListSettings,
    /// The language the user reads; `None` for the one of the device they
    /// are on.
    pub language: Option<Locale>,
}

fn projection<E: Executor>() -> Projection<E, SettingsState> {
    Projection::new::<Settings>()
        .handler(on_list_changed())
        .handler(on_language_changed())
        .strict()
}

#[evento::handler]
async fn on_list_changed(
    event: Event<ListChanged>,
    settings: &mut SettingsState,
) -> anyhow::Result<()> {
    let data = &event.data;
    settings.id = event.aggregate_id.to_owned();
    settings.list = ListSettings {
        servings: data.servings,
        meals: data.meals,
        sources: data.sources,
    };
    Ok(())
}

#[evento::handler]
async fn on_language_changed(
    event: Event<LanguageChanged>,
    settings: &mut SettingsState,
) -> anyhow::Result<()> {
    settings.id = event.aggregate_id.to_owned();
    // A language the api no longer speaks is one the user did not choose.
    settings.language = event.data.language.as_deref().and_then(Locale::from_tag);
    Ok(())
}

/// The settings of `user_id` as their stream leaves them, the defaults for
/// a user who changed none.
pub async fn load<E: Executor>(executor: &E, user_id: &str) -> anyhow::Result<SettingsState> {
    let settings = projection().load(user_id).execute(executor).await?;
    Ok(settings.unwrap_or_default())
}

/// Sets how the list of `user_id` is generated; the same as it stands
/// writes nothing.
pub async fn set_list<E: Executor>(
    executor: &E,
    user_id: &str,
    list: ListSettings,
) -> ApiResult<()> {
    write(executor, user_id, |settings, write| {
        if settings.list == list {
            return false;
        }
        write.event(&ListChanged {
            servings: list.servings,
            meals: list.meals,
            sources: list.sources,
        });
        true
    })
    .await
}

/// Sets the language `user_id` reads, `None` for the one of their device;
/// the same as it stands writes nothing.
pub async fn set_language<E: Executor>(
    executor: &E,
    user_id: &str,
    language: Option<Locale>,
) -> ApiResult<()> {
    write(executor, user_id, |settings, write| {
        if settings.language == language {
            return false;
        }
        write.event(&LanguageChanged {
            language: language.map(|language| language.as_str().to_owned()),
        });
        true
    })
    .await
}

/// Appends what `change` adds to the settings of `user_id`, at the version
/// they were loaded at, 0 for a user who changed none yet; `change` answers
/// `false` when there is nothing to write. A write another one got ahead of
/// is made again over the settings as they now stand.
async fn write<E: Executor>(
    executor: &E,
    user_id: &str,
    change: impl Fn(&SettingsState, &mut WriteBuilder) -> bool,
) -> ApiResult<()> {
    for _ in 0..WRITE_ATTEMPTS {
        let settings = load(executor, user_id).await?;
        let mut write = evento::append(user_id);
        write.original_version(settings.aggregate_version()?);
        if !change(&settings, &mut write) {
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
