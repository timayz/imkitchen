//! `/v1/plats/*`: the plats a user composes of their recipes.
//!
//! A plat is a composed dish: a name, and the recipes it is made of, in the
//! order they are served. Like them it is its owner's alone: every route
//! takes [`Session`] and acts on the plats of the user it signs in; someone
//! else's plat is answered as one that does not exist (`plat_not_found`),
//! read or written, and a plat is only ever composed of its owner's
//! recipes.
//!
//! A plat names its recipes and holds nothing of them: it answers each as
//! the recipe now stands. A recipe that is deleted leaves the plats it was
//! part of ([`subscription`]), a moment after the fact; a plat left with no
//! recipe stays, for its owner to recompose or delete.
//!
//! One plat is read from its stream, and its recipes from theirs, so it is
//! as the last command left it. The list is read from a table a
//! subscription fills from the streams, a moment behind them.

mod model;
mod read_model;

use std::collections::{HashMap, HashSet};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use timada_core::page::Cursor;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use validator::{Validate, ValidationError};

pub use self::model::PLAT_NOT_FOUND;
use self::model::{Composition, MAX_RECIPES, PlatState};
pub use self::read_model::{SUBSCRIPTION, subscription};
use crate::auth::Session;
use crate::db::Db;
use crate::error::{ApiResult, Body, Params, trimmed, trimmed_opt, valid_cursor};
use crate::openapi::PLATS;
use crate::recipes::{self, RecipeSummary};
use crate::server::AppState;

/// How many plats a page of the list holds when the request does not say.
const PAGE: u32 = 20;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create, list))
        .routes(routes!(plat, edit, delete))
}

fn no_duplicate(recipe_ids: &[String]) -> Result<(), ValidationError> {
    let mut seen = HashSet::new();
    if recipe_ids.iter().all(|id| seen.insert(id)) {
        Ok(())
    } else {
        Err(ValidationError::new("duplicate"))
    }
}

/// Everything a plat is: what composing one takes, and what recomposing
/// one sets it to.
#[derive(Deserialize, Validate, ToSchema)]
struct PlatBody {
    #[serde(deserialize_with = "trimmed")]
    #[validate(length(min = 1, max = 200))]
    #[schema(example = "Sunday roast")]
    name: String,
    /// The recipes it is made of, in the order they are served: recipes of
    /// the user, none twice.
    #[validate(
        length(min = 1, max = MAX_RECIPES, code = "count"),
        custom(function = "no_duplicate")
    )]
    recipe_ids: Vec<String>,
}

impl From<PlatBody> for Composition {
    fn from(body: PlatBody) -> Self {
        Self {
            name: body.name,
            recipe_ids: body.recipe_ids,
        }
    }
}

/// A plat as its owner is told of it. Moments are unix seconds.
#[derive(Serialize, ToSchema)]
pub struct Plat {
    id: String,
    name: String,
    /// The recipes it is made of, in the order they are served, each as the
    /// list of the recipes shows it.
    recipes: Vec<RecipeSummary>,
    created_at: u64,
    updated_at: u64,
}

/// A page of a user's plats.
#[derive(Serialize, ToSchema)]
struct PlatList {
    plats: Vec<Plat>,
    /// What `after` takes to read the page that follows; left out on the
    /// last one.
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<String>,
}

/// Which of the user's plats to list.
#[derive(Deserialize, Validate, IntoParams)]
#[into_params(parameter_in = Query)]
struct Listing {
    /// Only the plats whose name this is found in, whatever the case and
    /// the accents.
    #[serde(default, deserialize_with = "trimmed_opt")]
    #[validate(length(max = 100))]
    q: Option<String>,
    /// The `next` of the page before; left out for the first page.
    #[validate(custom(function = "valid_cursor"))]
    after: Option<String>,
    /// How many plats at most, 20 when left out.
    #[validate(range(min = 1, max = 100))]
    limit: Option<u32>,
}

/// The plat `id` of `owner_id` with its recipes, each as its stream leaves
/// it. One that was deleted, and is not taken out of the plat yet, is left
/// out.
async fn read(state: &AppState, owner_id: &str, id: &str) -> ApiResult<Plat> {
    let plat: PlatState = model::load_own(&state.executor, owner_id, id).await?;
    Ok(shown(&state.executor, plat).await?)
}

/// `plat` as its owner is told of it: with its recipes, each as its stream
/// leaves it.
async fn shown<E: evento::Executor>(executor: &E, plat: PlatState) -> anyhow::Result<Plat> {
    let mut recipes = Vec::new();
    for recipe_id in &plat.composition.recipe_ids {
        recipes.extend(recipes::own(executor, &plat.owner_id, recipe_id).await?);
    }
    Ok(Plat {
        id: plat.id,
        name: plat.composition.name,
        recipes,
        created_at: plat.created_at,
        updated_at: plat.updated_at,
    })
}

/// The plat `id` as its owner is told of it, when it is one of
/// `owner_id`'s: `None` for one that does not exist, was deleted or is
/// someone else's.
pub async fn own<E: evento::Executor>(
    executor: &E,
    owner_id: &str,
    id: &str,
) -> anyhow::Result<Option<Plat>> {
    let plat = model::load(executor, id).await?;
    match plat.filter(|plat| !plat.deleted && plat.owner_id == owner_id) {
        Some(plat) => Ok(Some(shown(executor, plat).await?)),
        None => Ok(None),
    }
}

/// Whether `id` is a plat of `owner_id`: one that exists and was not
/// deleted.
pub async fn is_own<E: evento::Executor>(
    executor: &E,
    owner_id: &str,
    id: &str,
) -> anyhow::Result<bool> {
    let plat = model::load(executor, id).await?;
    Ok(plat.is_some_and(|plat| !plat.deleted && plat.owner_id == owner_id))
}

/// The ids of the recipes the plat `id` is made of, in the order they are
/// served, when it is one of `owner_id`'s: `None` for one that does not
/// exist, was deleted or is someone else's.
pub async fn recipe_ids_of<E: evento::Executor>(
    executor: &E,
    owner_id: &str,
    id: &str,
) -> anyhow::Result<Option<Vec<String>>> {
    let plat = model::load(executor, id).await?;
    Ok(plat
        .filter(|plat| !plat.deleted && plat.owner_id == owner_id)
        .map(|plat| plat.composition.recipe_ids))
}

/// Whose the plat `id` is, or was when it was deleted; `None` for one that
/// was never composed.
pub async fn owner_of<E: evento::Executor>(
    executor: &E,
    id: &str,
) -> anyhow::Result<Option<String>> {
    let plat = model::load(executor, id).await?;
    Ok(plat.map(|plat| plat.owner_id))
}

/// Every plat of `owner_id` this node's list knows, each with the ids of
/// its recipes, in no order: one just composed is there a moment later.
pub async fn composed_by(db: &Db, owner_id: &str) -> anyhow::Result<Vec<(String, Vec<String>)>> {
    let plats = read_model::composed_by(db, owner_id).await?;
    Ok(plats
        .into_iter()
        .map(|plat| (plat.id, plat.recipe_ids.0))
        .collect())
}

/// Composes a plat of recipes of the user
///
/// Refused with `validation_failed` when one of `recipe_ids` is not a
/// recipe of the user: its field says `recipe_not_found`.
#[utoipa::path(
    post,
    path = "/plats",
    tag = PLATS,
    security(("session" = [])),
    request_body = PlatBody,
    responses((status = CREATED, description = "The plat is composed", body = Plat))
)]
async fn create(
    State(state): State<AppState>,
    session: Session,
    Body(form): Body<PlatBody>,
) -> ApiResult<(StatusCode, Json<Plat>)> {
    let owner_id = &session.user.id;
    let id = model::create(&state.executor, owner_id, form.into()).await?;
    let plat = read(&state, owner_id, &id).await?;
    Ok((StatusCode::CREATED, Json(plat)))
}

/// The plats of the user, the most recent first
///
/// A page at a time: `next` is what `after` takes to read the following
/// one. With `q`, only the plats it is found in the name of. A plat just
/// composed, recomposed or deleted is in the list a moment later, and so
/// is what changed in its recipes.
#[utoipa::path(
    get,
    path = "/plats",
    tag = PLATS,
    security(("session" = [])),
    params(Listing),
    responses((status = OK, body = PlatList))
)]
async fn list(
    State(state): State<AppState>,
    session: Session,
    Params(listing): Params<Listing>,
) -> ApiResult<Json<PlatList>> {
    let limit = listing.limit.unwrap_or(PAGE) as usize;
    let after = listing.after.as_deref().and_then(Cursor::parse);
    // One more than shown: whether there is a page beyond.
    let mut rows = read_model::plats_of(
        &state.db,
        &session.user.id,
        listing.q.as_deref(),
        after.as_ref(),
        limit as i64 + 1,
    )
    .await?;
    let more = rows.len() > limit;
    rows.truncate(limit);
    let next = rows.last().filter(|_| more).map(|row| row.cursor().param());

    // The recipes of the whole page at once, as the list of the recipes
    // knows them.
    let recipe_ids: HashSet<&String> = rows.iter().flat_map(|row| &row.recipe_ids.0).collect();
    let recipe_ids: Vec<String> = recipe_ids.into_iter().cloned().collect();
    let recipes: HashMap<String, RecipeSummary> = recipes::listed(&state.db, &recipe_ids)
        .await?
        .into_iter()
        .map(|recipe| (recipe.id.clone(), recipe))
        .collect();
    let plats = rows
        .into_iter()
        .map(|row| Plat {
            id: row.id,
            name: row.name,
            recipes: row
                .recipe_ids
                .0
                .iter()
                .filter_map(|id| recipes.get(id).cloned())
                .collect(),
            created_at: row.created_at as u64,
            updated_at: row.updated_at as u64,
        })
        .collect();
    Ok(Json(PlatList { plats, next }))
}

/// A plat of the user
#[utoipa::path(
    get,
    path = "/plats/{id}",
    tag = PLATS,
    security(("session" = [])),
    params(("id" = String, Path, description = "The plat's id")),
    responses((status = OK, body = Plat))
)]
async fn plat(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> ApiResult<Json<Plat>> {
    Ok(Json(read(&state, &session.user.id, &id).await?))
}

/// Recomposes a plat of the user
///
/// The plat is set to what the body says, as a whole: its name, and its
/// recipes in the order given. Refused as composing one is.
#[utoipa::path(
    put,
    path = "/plats/{id}",
    tag = PLATS,
    security(("session" = [])),
    params(("id" = String, Path, description = "The plat's id")),
    request_body = PlatBody,
    responses((status = NO_CONTENT, description = "The plat is recomposed"))
)]
async fn edit(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Body(form): Body<PlatBody>,
) -> ApiResult<StatusCode> {
    model::edit(&state.executor, &session.user.id, &id, form.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes a plat of the user
///
/// The recipes it was made of stay.
#[utoipa::path(
    delete,
    path = "/plats/{id}",
    tag = PLATS,
    security(("session" = [])),
    params(("id" = String, Path, description = "The plat's id")),
    responses((status = NO_CONTENT, description = "The plat is deleted"))
)]
async fn delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    model::delete(&state.executor, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
