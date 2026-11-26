//! `/v1/settings/*`: what a user says of how their list is generated and of
//! the language they read.
//!
//! A user has one set of settings, their own: every route takes [`Session`]
//! and acts on the settings of the user it signs in. They are there from
//! the start, as the defaults, and each part of them is set as a whole.
//!
//! They are read from their stream, so they are as the last command left
//! them, here and for what generates the list ([`of`]).

mod model;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use imkitchen_aggregates::settings::ListSources;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use validator::Validate;

pub use self::model::ListSettings;
use self::model::SettingsState;
use crate::auth::Session;
use crate::error::{ApiResult, Body};
use crate::locale::Locale;
use crate::openapi::SETTINGS;
use crate::server::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(settings))
        .routes(routes!(set_list))
        .routes(routes!(set_language))
}

/// How `user_id` wants their list generated, as their last command left
/// it: the defaults for a user who said nothing.
pub async fn of<E: evento::Executor>(executor: &E, user_id: &str) -> anyhow::Result<ListSettings> {
    Ok(model::load(executor, user_id).await?.list)
}

/// What the meals of a generated list are drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum Sources {
    /// The user's plats and their recipes.
    PlatsAndRecipes,
    /// Only the user's plats, as composed.
    Plats,
    /// Only the user's recipes.
    Recipes,
}

impl From<Sources> for ListSources {
    fn from(sources: Sources) -> Self {
        match sources {
            Sources::PlatsAndRecipes => Self::PlatsAndRecipes,
            Sources::Plats => Self::Plats,
            Sources::Recipes => Self::Recipes,
        }
    }
}

impl From<ListSources> for Sources {
    fn from(sources: ListSources) -> Self {
        match sources {
            ListSources::PlatsAndRecipes => Self::PlatsAndRecipes,
            ListSources::Plats => Self::Plats,
            ListSources::Recipes => Self::Recipes,
        }
    }
}

/// How the user's list is generated: what setting it takes, and what the
/// settings answer of it.
#[derive(Serialize, Deserialize, Validate, ToSchema)]
struct List {
    /// How many people the list is cooked for: what its recipes are scaled
    /// to.
    #[validate(range(min = 1, max = 1000))]
    #[schema(example = 4)]
    servings: u32,
    /// How many meals a generated list holds.
    #[validate(range(min = 1, max = 31))]
    #[schema(example = 7)]
    meals: u32,
    sources: Sources,
}

impl From<List> for ListSettings {
    fn from(list: List) -> Self {
        Self {
            servings: list.servings,
            meals: list.meals,
            sources: list.sources.into(),
        }
    }
}

/// The settings of a user.
#[derive(Serialize, ToSchema)]
struct Settings {
    list: List,
    /// The language the user reads; left out for the one of the device
    /// they are on.
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<Locale>,
}

impl From<SettingsState> for Settings {
    fn from(settings: SettingsState) -> Self {
        Self {
            list: List {
                servings: settings.list.servings,
                meals: settings.list.meals,
                sources: settings.list.sources.into(),
            },
            language: settings.language,
        }
    }
}

#[derive(Deserialize, Validate, ToSchema)]
struct Language {
    /// One of the languages the api speaks; left out, the one of the device
    /// the user is on.
    language: Option<Locale>,
}

/// The settings of the user
///
/// A user who set nothing has the defaults: a list of 7 meals for 4, drawn
/// from their plats and their recipes, and no language.
#[utoipa::path(
    get,
    path = "/settings",
    tag = SETTINGS,
    security(("session" = [])),
    responses((status = OK, body = Settings))
)]
async fn settings(State(state): State<AppState>, session: Session) -> ApiResult<Json<Settings>> {
    let settings = model::load(&state.executor, &session.user.id).await?;
    Ok(Json(settings.into()))
}

/// Sets how the list of the user is generated
///
/// As a whole: how many it is cooked for, how many meals it holds and what
/// they are drawn from. The list as it stands is left as it is: the next
/// one generated follows.
#[utoipa::path(
    put,
    path = "/settings/list",
    tag = SETTINGS,
    security(("session" = [])),
    request_body = List,
    responses((status = NO_CONTENT, description = "The list is generated this way from now on"))
)]
async fn set_list(
    State(state): State<AppState>,
    session: Session,
    Body(form): Body<List>,
) -> ApiResult<StatusCode> {
    model::set_list(&state.executor, &session.user.id, form.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Sets the language the user reads
///
/// On every device they sign in on. Without one, each device goes by its
/// own.
#[utoipa::path(
    put,
    path = "/settings/language",
    tag = SETTINGS,
    security(("session" = [])),
    request_body = Language,
    responses((status = NO_CONTENT, description = "The language is set"))
)]
async fn set_language(
    State(state): State<AppState>,
    session: Session,
    Body(form): Body<Language>,
) -> ApiResult<StatusCode> {
    model::set_language(&state.executor, &session.user.id, form.language).await?;
    Ok(StatusCode::NO_CONTENT)
}
