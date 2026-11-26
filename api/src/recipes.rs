//! `/v1/recipes/*`: a user's own recipes.
//!
//! A recipe is its owner's alone: there is no sharing, no chef and no public
//! profile. Every route takes [`Session`] and acts on the recipes of the
//! user it signs in; someone else's recipe is answered as one that does not
//! exist (`recipe_not_found`), read or written.
//!
//! A recipe is written by hand or imported from a page of the web
//! ([`import`]); once it is there, nothing tells the two apart.
//!
//! One recipe is read from its stream, so it is as the last command left
//! it. The list is read from a table a subscription fills from the streams
//! ([`subscription`]), a moment behind them.

mod import;
mod model;
mod read_model;

use axum::Json;
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use timada_core::page::Cursor;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouterExt};
use utoipa_axum::routes;
use validator::{Validate, ValidationError};

use self::import::valid_page_url;
pub use self::import::{NO_RECIPE_ON_PAGE, PAGE_UNREACHABLE, PageError, Pages, Web};
pub use self::model::RECIPE_NOT_FOUND;
use self::model::{Content, RecipeState};
use self::read_model::RecipeRow;
pub use self::read_model::{SUBSCRIPTION, subscription};
use crate::auth::Session;
use crate::error::{ApiResult, Body, Params, or_default, trimmed, trimmed_opt};
use crate::openapi::RECIPES;
use crate::server::AppState;

/// How many recipes a page of the list holds when the request does not say.
const PAGE: u32 = 20;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create, list))
        // A page sent along weighs more than a body usually may; twice
        // what it holds, for what JSON adds to it.
        .routes(routes!(import).layer(DefaultBodyLimit::max(2 * import::PAGE_MAX)))
        .routes(routes!(recipe, edit, delete))
}

/// What a recipe takes of one thing.
#[derive(Serialize, Deserialize, Validate, ToSchema)]
struct Ingredient {
    #[serde(deserialize_with = "trimmed")]
    #[validate(length(min = 1, max = 200))]
    #[schema(example = "flour")]
    name: String,
    /// How much of it, in `unit`. Left out when the recipe does not say
    /// ("salt").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 0.001, max = 1000000.0))]
    #[schema(example = 250)]
    quantity: Option<f64>,
    /// What `quantity` counts ("g", "tbsp"). Left out for the thing itself
    /// ("2 eggs").
    #[serde(
        default,
        deserialize_with = "trimmed_opt",
        skip_serializing_if = "Option::is_none"
    )]
    #[validate(length(max = 32))]
    #[schema(example = "g")]
    unit: Option<String>,
}

/// One step of a recipe.
#[derive(Serialize, Deserialize, Validate, ToSchema)]
struct Instruction {
    #[serde(deserialize_with = "trimmed")]
    #[validate(length(min = 1, max = 4000))]
    text: String,
}

/// Everything a recipe says: what writing one takes, and what rewriting one
/// sets it to.
#[derive(Deserialize, Validate, ToSchema)]
struct RecipeBody {
    #[serde(deserialize_with = "trimmed")]
    #[validate(length(min = 1, max = 200))]
    title: String,
    #[serde(default, deserialize_with = "trimmed_opt")]
    #[validate(length(max = 2000))]
    description: Option<String>,
    #[serde(default, deserialize_with = "or_default")]
    #[validate(length(max = 100, code = "too_many"), nested)]
    ingredients: Vec<Ingredient>,
    /// The steps, in the order they are made.
    #[serde(default, deserialize_with = "or_default")]
    #[validate(length(max = 100, code = "too_many"), nested)]
    instructions: Vec<Instruction>,
    /// Minutes of preparation; left out, 0: the recipe says none.
    #[serde(default, deserialize_with = "or_default")]
    #[validate(range(max = 10080))]
    prep_minutes: u32,
    /// Minutes of cooking; left out, 0: the recipe says none.
    #[serde(default, deserialize_with = "or_default")]
    #[validate(range(max = 10080))]
    cook_minutes: u32,
    /// How many people it feeds as written.
    #[validate(range(min = 1, max = 1000))]
    servings: u32,
}

impl From<RecipeBody> for Content {
    fn from(body: RecipeBody) -> Self {
        Self {
            title: body.title,
            description: body.description.unwrap_or_default(),
            ingredients: body
                .ingredients
                .into_iter()
                .map(|one| imkitchen_aggregates::recipe::Ingredient {
                    name: one.name,
                    quantity: one.quantity,
                    unit: one.unit,
                })
                .collect(),
            instructions: body.instructions.into_iter().map(|one| one.text).collect(),
            prep_minutes: body.prep_minutes,
            cook_minutes: body.cook_minutes,
            servings: body.servings,
        }
    }
}

/// Where to import a recipe from.
#[derive(Deserialize, Validate, ToSchema)]
struct ImportBody {
    /// The address of a page that holds a recipe: one that describes it
    /// the way schema.org says to (`Recipe`, as JSON-LD or microdata), which
    /// most recipe sites do, or an article that shows its ingredients and
    /// its steps under headings that say so.
    #[serde(deserialize_with = "trimmed")]
    #[validate(length(max = 2000), custom(function = "valid_page_url"))]
    #[schema(
        example = "https://www.marmiton.org/recettes/recette_pancakes-faciles-et-rapides_81518.aspx"
    )]
    url: String,
    /// The page at `url`, as the browser of the client got it. With it the
    /// api reads the recipe on what is sent and fetches nothing: the way
    /// to import from a site that only answers the browsers of people.
    #[serde(default)]
    #[validate(length(max = 4194304))]
    html: Option<String>,
}

/// A recipe as its owner is told of it. Times are minutes, 0 when the
/// recipe says none; moments are unix seconds.
#[derive(Serialize, ToSchema)]
struct Recipe {
    id: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    ingredients: Vec<Ingredient>,
    /// The steps, in the order they are made.
    instructions: Vec<Instruction>,
    prep_minutes: u32,
    cook_minutes: u32,
    servings: u32,
    created_at: u64,
    updated_at: u64,
}

impl From<RecipeState> for Recipe {
    fn from(recipe: RecipeState) -> Self {
        let content = recipe.content;
        Self {
            id: recipe.id,
            title: content.title,
            description: Some(content.description).filter(|text| !text.is_empty()),
            ingredients: content
                .ingredients
                .into_iter()
                .map(|one| Ingredient {
                    name: one.name,
                    quantity: one.quantity,
                    unit: one.unit,
                })
                .collect(),
            instructions: content
                .instructions
                .into_iter()
                .map(|text| Instruction { text })
                .collect(),
            prep_minutes: content.prep_minutes,
            cook_minutes: content.cook_minutes,
            servings: content.servings,
            created_at: recipe.created_at,
            updated_at: recipe.updated_at,
        }
    }
}

/// What the list shows of a recipe: everything but its ingredients and its
/// steps.
#[derive(Serialize, ToSchema)]
struct RecipeSummary {
    id: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    prep_minutes: i64,
    cook_minutes: i64,
    servings: i64,
    created_at: i64,
    updated_at: i64,
}

impl From<RecipeRow> for RecipeSummary {
    fn from(row: RecipeRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            description: Some(row.description).filter(|text| !text.is_empty()),
            prep_minutes: row.prep_minutes,
            cook_minutes: row.cook_minutes,
            servings: row.servings,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// A page of a user's recipes.
#[derive(Serialize, ToSchema)]
struct RecipeList {
    recipes: Vec<RecipeSummary>,
    /// What `after` takes to read the page that follows; left out on the
    /// last one.
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<String>,
}

fn valid_cursor(after: &str) -> Result<(), ValidationError> {
    Cursor::parse(after)
        .map(drop)
        .ok_or(ValidationError::new("invalid_cursor"))
}

/// Which of the user's recipes to list.
#[derive(Deserialize, Validate, IntoParams)]
#[into_params(parameter_in = Query)]
struct Listing {
    /// Only the recipes this is found in: their title or one of their
    /// ingredients, whatever the case and the accents.
    #[serde(default, deserialize_with = "trimmed_opt")]
    #[validate(length(max = 100))]
    q: Option<String>,
    /// The `next` of the page before; left out for the first page.
    #[validate(custom(function = "valid_cursor"))]
    after: Option<String>,
    /// How many recipes at most, 20 when left out.
    #[validate(range(min = 1, max = 100))]
    limit: Option<u32>,
}

/// Writes a recipe of the user
#[utoipa::path(
    post,
    path = "/recipes",
    tag = RECIPES,
    security(("session" = [])),
    request_body = RecipeBody,
    responses((status = CREATED, description = "The recipe is written", body = Recipe))
)]
async fn create(
    State(state): State<AppState>,
    session: Session,
    Body(form): Body<RecipeBody>,
) -> ApiResult<(StatusCode, Json<Recipe>)> {
    let owner_id = &session.user.id;
    let id = model::create(&state.executor, owner_id, form.into()).await?;
    let recipe = model::load_own(&state.executor, owner_id, &id).await?;
    Ok((StatusCode::CREATED, Json(recipe.into())))
}

/// Imports a recipe from a page of the web
///
/// The recipe the page holds becomes one of the user's, theirs to
/// rewrite like one they wrote; nothing links it to the page. What the page
/// does not say is left out (the times), or taken as usual (4 servings).
/// A site that refuses the api answers the browser of the client: the
/// page it got is sent as `html`, and the api reads that instead of
/// fetching. Refused with `page_unreachable` when the page cannot be read, and with
/// `no_recipe_on_page` when no recipe is read on it.
#[utoipa::path(
    post,
    path = "/recipes/import",
    tag = RECIPES,
    security(("session" = [])),
    request_body = ImportBody,
    responses((status = CREATED, description = "The recipe is imported", body = Recipe))
)]
async fn import(
    State(state): State<AppState>,
    session: Session,
    Body(form): Body<ImportBody>,
) -> ApiResult<(StatusCode, Json<Recipe>)> {
    let owner_id = &session.user.id;
    let content = import::content(state.pages.as_ref(), &form.url, form.html).await?;
    let id = model::create(&state.executor, owner_id, content).await?;
    let recipe = model::load_own(&state.executor, owner_id, &id).await?;
    Ok((StatusCode::CREATED, Json(recipe.into())))
}

/// The recipes of the user, the most recent first
///
/// A page at a time: `next` is what `after` takes to read the following
/// one. With `q`, only the recipes it is found in. A recipe just written,
/// rewritten or deleted is in the list a moment later.
#[utoipa::path(
    get,
    path = "/recipes",
    tag = RECIPES,
    security(("session" = [])),
    params(Listing),
    responses((status = OK, body = RecipeList))
)]
async fn list(
    State(state): State<AppState>,
    session: Session,
    Params(listing): Params<Listing>,
) -> ApiResult<Json<RecipeList>> {
    let limit = listing.limit.unwrap_or(PAGE) as usize;
    let after = listing.after.as_deref().and_then(Cursor::parse);
    // One more than shown: whether there is a page beyond.
    let mut rows = read_model::recipes_of(
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
    Ok(Json(RecipeList {
        recipes: rows.into_iter().map(RecipeSummary::from).collect(),
        next,
    }))
}

/// A recipe of the user
#[utoipa::path(
    get,
    path = "/recipes/{id}",
    tag = RECIPES,
    security(("session" = [])),
    params(("id" = String, Path, description = "The recipe's id")),
    responses((status = OK, body = Recipe))
)]
async fn recipe(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> ApiResult<Json<Recipe>> {
    let recipe = model::load_own(&state.executor, &session.user.id, &id).await?;
    Ok(Json(recipe.into()))
}

/// Rewrites a recipe of the user
///
/// The recipe is set to what the body says, as a whole: what the body
/// leaves out is no longer in it.
#[utoipa::path(
    put,
    path = "/recipes/{id}",
    tag = RECIPES,
    security(("session" = [])),
    params(("id" = String, Path, description = "The recipe's id")),
    request_body = RecipeBody,
    responses((status = NO_CONTENT, description = "The recipe is rewritten"))
)]
async fn edit(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Body(form): Body<RecipeBody>,
) -> ApiResult<StatusCode> {
    model::edit(&state.executor, &session.user.id, &id, form.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes a recipe of the user
#[utoipa::path(
    delete,
    path = "/recipes/{id}",
    tag = RECIPES,
    security(("session" = [])),
    params(("id" = String, Path, description = "The recipe's id")),
    responses((status = NO_CONTENT, description = "The recipe is deleted"))
)]
async fn delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    model::delete(&state.executor, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
