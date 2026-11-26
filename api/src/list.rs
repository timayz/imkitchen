//! `/v1/list/*`: the meals a user plans to cook, generated from their
//! recipes and their plats or picked by hand.
//!
//! A user has one list, their own: every route takes [`Session`] and acts
//! on the list of the user it signs in. It is there from the start, empty.
//! Each of its entries is a meal: a recipe or a plat of the user, cooked
//! for so many people. The same one may be planned more than once, so an
//! entry has an id of its own.
//!
//! Generating draws the list anew, as the user's settings say
//! (`crate::settings`), from what the lists of their recipes and of their
//! plats show.
//!
//! The list names its meals and holds nothing of them: it answers each as
//! the recipe or the plat now stands. One that is deleted leaves the list
//! ([`subscription`]), a moment after the fact. The list is read from its
//! stream, and its meals from theirs, so it is as the last command left
//! it.

mod model;
mod read_model;

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use imkitchen_aggregates::list::Meal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use validator::Validate;

use self::model::Pick;
pub use self::model::{LIST_ENTRY_NOT_FOUND, LIST_FULL, NOTHING_TO_GENERATE};
pub use self::read_model::{SUBSCRIPTION, subscription};
use crate::auth::Session;
use crate::error::{ApiError, ApiResult, Body, FieldError};
use crate::openapi::LIST;
use crate::plats::{self, Plat};
use crate::recipes::{self, RecipeSummary};
use crate::server::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(generate))
        .routes(routes!(add))
        .routes(routes!(replace, remove))
}

/// A meal to plan: a recipe of the user or one of their plats, one of the
/// two.
#[derive(Deserialize, Validate, ToSchema)]
struct EntryBody {
    /// The recipe to cook, when the meal is a recipe.
    recipe_id: Option<String>,
    /// The plat to cook, when the meal is a plat.
    plat_id: Option<String>,
    /// How many people it is cooked for; left out, as many as the user's
    /// settings say.
    #[validate(range(min = 1, max = 1000))]
    #[schema(example = 4)]
    servings: Option<u32>,
}

impl TryFrom<EntryBody> for Pick {
    type Error = ApiError;

    fn try_from(body: EntryBody) -> ApiResult<Self> {
        let refused = |field: &str, code: &'static str| {
            ApiError::Invalid(vec![FieldError {
                field: field.to_owned(),
                code: code.into(),
                params: BTreeMap::new(),
            }])
        };
        let meal = match (body.recipe_id, body.plat_id) {
            (Some(id), None) => Meal::Recipe(id),
            (None, Some(id)) => Meal::Plat(id),
            (None, None) => return Err(refused("recipe_id", "required")),
            (Some(_), Some(_)) => return Err(refused("plat_id", "exclusive")),
        };
        Ok(Self {
            meal,
            servings: body.servings,
        })
    }
}

/// A meal of the list: `recipe` or `plat` says what is cooked, as it now
/// stands.
#[derive(Serialize, ToSchema)]
struct Entry {
    id: String,
    /// How many people it is cooked for.
    servings: u32,
    /// The recipe, as the list of the recipes shows it, when the meal is
    /// one.
    #[serde(skip_serializing_if = "Option::is_none")]
    recipe: Option<RecipeSummary>,
    /// The plat, with its recipes, when the meal is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    plat: Option<Plat>,
}

/// The list of a user.
#[derive(Serialize, ToSchema)]
struct List {
    /// The meals, in the order they were planned.
    entries: Vec<Entry>,
}

/// `entry` of the list of `owner_id` as it is answered, its meal as its
/// stream leaves it. `None` when the meal was deleted and is not taken off
/// the list yet.
async fn shown<E: evento::Executor>(
    executor: &E,
    owner_id: &str,
    entry: imkitchen_aggregates::list::Entry,
) -> anyhow::Result<Option<Entry>> {
    let (recipe, plat) = match &entry.meal {
        Meal::Recipe(id) => (recipes::own(executor, owner_id, id).await?, None),
        Meal::Plat(id) => (None, plats::own(executor, owner_id, id).await?),
    };
    Ok((recipe.is_some() || plat.is_some()).then_some(Entry {
        id: entry.id,
        servings: entry.servings,
        recipe,
        plat,
    }))
}

/// The list of `owner_id` as their last command left it.
async fn read(state: &AppState, owner_id: &str) -> ApiResult<List> {
    let list = model::load(&state.executor, owner_id).await?;
    let mut entries = Vec::new();
    for entry in list.entries {
        entries.extend(shown(&state.executor, owner_id, entry).await?);
    }
    Ok(List { entries })
}

/// The list of the user
///
/// The meals they plan to cook, in the order they were planned. Empty
/// until they generate it or pick a meal.
#[utoipa::path(
    get,
    path = "/list",
    tag = LIST,
    security(("session" = [])),
    responses((status = OK, body = List))
)]
async fn list(State(state): State<AppState>, session: Session) -> ApiResult<Json<List>> {
    Ok(Json(read(&state, &session.user.id).await?))
}

/// Generates the list of the user
///
/// Drawn at random as their settings say, in place of what it held: so
/// many meals, from their plats, their recipes or both, each cooked for so
/// many. No meal is there twice, and there are fewer than asked when the
/// user has not as many. A recipe or a plat just written is drawn from a
/// moment later. Refused with `nothing_to_generate` when there is nothing
/// to draw.
#[utoipa::path(
    post,
    path = "/list/generate",
    tag = LIST,
    security(("session" = [])),
    responses((status = CREATED, description = "The list is generated", body = List))
)]
async fn generate(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<(StatusCode, Json<List>)> {
    let owner_id = &session.user.id;
    model::generate(&state.executor, &state.db, owner_id).await?;
    Ok((StatusCode::CREATED, Json(read(&state, owner_id).await?)))
}

/// Puts a meal on the list of the user
///
/// After those it holds. Refused with `validation_failed` when the meal is
/// not a recipe or a plat of the user (`recipe_not_found`,
/// `plat_not_found` under its field), and with `list_full` when the list
/// holds 50 meals.
#[utoipa::path(
    post,
    path = "/list/entries",
    tag = LIST,
    security(("session" = [])),
    request_body = EntryBody,
    responses((status = CREATED, description = "The meal is on the list", body = Entry))
)]
async fn add(
    State(state): State<AppState>,
    session: Session,
    Body(form): Body<EntryBody>,
) -> ApiResult<(StatusCode, Json<Entry>)> {
    let owner_id = &session.user.id;
    let entry = model::add(&state.executor, owner_id, form.try_into()?).await?;
    // Deleted since: it is not on the list for long.
    let entry = shown(&state.executor, owner_id, entry)
        .await?
        .ok_or(ApiError::refused(
            StatusCode::NOT_FOUND,
            LIST_ENTRY_NOT_FOUND,
        ))?;
    Ok((StatusCode::CREATED, Json(entry)))
}

/// Replaces a meal of the list of the user
///
/// The entry is set to what the body says, as a whole, in the place it
/// has. Refused as putting a meal on the list is.
#[utoipa::path(
    put,
    path = "/list/entries/{id}",
    tag = LIST,
    security(("session" = [])),
    params(("id" = String, Path, description = "The entry's id")),
    request_body = EntryBody,
    responses((status = NO_CONTENT, description = "The meal is replaced"))
)]
async fn replace(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Body(form): Body<EntryBody>,
) -> ApiResult<StatusCode> {
    model::replace(&state.executor, &session.user.id, &id, form.try_into()?).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Takes a meal off the list of the user
///
/// Its recipe or its plat stays.
#[utoipa::path(
    delete,
    path = "/list/entries/{id}",
    tag = LIST,
    security(("session" = [])),
    params(("id" = String, Path, description = "The entry's id")),
    responses((status = NO_CONTENT, description = "The meal is off the list"))
)]
async fn remove(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    model::remove(&state.executor, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
