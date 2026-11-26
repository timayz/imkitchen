//! `/v1/groceries/*`: what a user has to buy to cook their list.
//!
//! A user has one set of groceries, their own: every route takes
//! [`Session`] and acts on the groceries of the user it signs in. They are
//! worked out from the list as it stands ([`items`]): the ingredients of
//! every meal, scaled to the people it is cooked for, the same ones
//! merged. Nothing of it is kept, so they are the list's at every read,
//! and its recipes' as they now stand.
//!
//! What the user says of an item is kept ([`model`]): that it is in the
//! basket, or that they buy another amount of it. It is said of the item
//! as the list then called for it, and holds for as long as it calls for
//! that much: a list change that leaves the item as it was leaves it
//! checked, one that changes how much is needed has it to buy again.

mod items;
mod model;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use validator::Validate;

use crate::auth::Session;
use crate::error::{ApiError, ApiResult, Body};
use crate::openapi::GROCERIES;
use crate::server::AppState;

/// The list calls for no such item: it never did, or no longer does.
pub const GROCERY_ITEM_NOT_FOUND: &str = "grocery_item_not_found";

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(groceries))
        .routes(routes!(check, uncheck))
        .routes(routes!(adjust, unadjust))
}

/// One thing to buy.
#[derive(Serialize, ToSchema)]
struct Item {
    /// The same for the same thing in the same unit, whatever the list
    /// holds.
    #[schema(example = "5e0f2c1d9a7b4e3f8a6c0b1d2e3f4a5b")]
    id: String,
    #[schema(example = "flour")]
    name: String,
    /// What `quantity` counts ("g", "tbsp"). Left out for the thing itself
    /// ("2 eggs").
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "g")]
    unit: Option<String>,
    /// How much the list calls for, in `unit`. Left out when no recipe
    /// says ("salt").
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 350)]
    quantity: Option<f64>,
    /// How much the user buys instead, when they said so.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 500)]
    adjusted_quantity: Option<f64>,
    /// In the basket.
    checked: bool,
}

/// The groceries of a user.
#[derive(Serialize, ToSchema)]
struct Groceries {
    /// What there is to buy, by name.
    items: Vec<Item>,
}

/// How much of an item the user buys.
#[derive(Deserialize, Validate, ToSchema)]
struct QuantityBody {
    /// In the unit of the item.
    #[validate(range(min = 0.001, max = 1000000.0))]
    #[schema(example = 500)]
    quantity: f64,
}

/// The item `id` of the groceries of `owner_id`, as the list now calls for
/// it.
async fn item(state: &AppState, owner_id: &str, id: &str) -> ApiResult<items::Item> {
    let items = items::of(&state.executor, owner_id).await?;
    items
        .into_iter()
        .find(|item| item.id == id)
        .ok_or(ApiError::refused(
            StatusCode::NOT_FOUND,
            GROCERY_ITEM_NOT_FOUND,
        ))
}

/// The groceries of the user
///
/// What their list takes, as it stands: the ingredients of every meal,
/// scaled to the people it is cooked for, by name. The same thing in the
/// same unit is one item, of the sum. Empty for an empty list.
#[utoipa::path(
    get,
    path = "/groceries",
    tag = GROCERIES,
    security(("session" = [])),
    responses((status = OK, body = Groceries))
)]
async fn groceries(State(state): State<AppState>, session: Session) -> ApiResult<Json<Groceries>> {
    let owner_id = &session.user.id;
    let items = items::of(&state.executor, owner_id).await?;
    let said = model::load(&state.executor, owner_id).await?;
    let items = items
        .into_iter()
        .map(|item| {
            let mark = said.mark(&item);
            Item {
                id: item.id,
                name: item.name,
                unit: item.unit,
                quantity: item.quantity,
                adjusted_quantity: mark.quantity,
                checked: mark.checked,
            }
        })
        .collect();
    Ok(Json(Groceries { items }))
}

/// Puts an item of the groceries in the basket
///
/// It stays checked for as long as the list calls for as much of it.
/// Refused with `grocery_item_not_found` when the list calls for no such
/// item.
#[utoipa::path(
    put,
    path = "/groceries/items/{id}/check",
    tag = GROCERIES,
    security(("session" = [])),
    params(("id" = String, Path, description = "The item's id")),
    responses((status = NO_CONTENT, description = "The item is checked"))
)]
async fn check(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let owner_id = &session.user.id;
    let item = item(&state, owner_id, &id).await?;
    model::check(&state.executor, owner_id, &item, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Takes an item of the groceries out of the basket
///
/// It is to buy again.
#[utoipa::path(
    delete,
    path = "/groceries/items/{id}/check",
    tag = GROCERIES,
    security(("session" = [])),
    params(("id" = String, Path, description = "The item's id")),
    responses((status = NO_CONTENT, description = "The item is unchecked"))
)]
async fn uncheck(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let owner_id = &session.user.id;
    let item = item(&state, owner_id, &id).await?;
    model::check(&state.executor, owner_id, &item, false).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Sets how much of an item the user buys
///
/// In place of what the list calls for, which is still answered
/// (`quantity`) next to it (`adjusted_quantity`). It holds for as long as
/// the list calls for as much. Checked or not, the item stays as it was.
#[utoipa::path(
    put,
    path = "/groceries/items/{id}/quantity",
    tag = GROCERIES,
    security(("session" = [])),
    params(("id" = String, Path, description = "The item's id")),
    request_body = QuantityBody,
    responses((status = NO_CONTENT, description = "The item is adjusted"))
)]
async fn adjust(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
    Body(form): Body<QuantityBody>,
) -> ApiResult<StatusCode> {
    let owner_id = &session.user.id;
    let item = item(&state, owner_id, &id).await?;
    model::adjust(&state.executor, owner_id, &item, Some(form.quantity)).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Has the user buy what the list calls for again
#[utoipa::path(
    delete,
    path = "/groceries/items/{id}/quantity",
    tag = GROCERIES,
    security(("session" = [])),
    params(("id" = String, Path, description = "The item's id")),
    responses((status = NO_CONTENT, description = "The item is as the list calls for it"))
)]
async fn unadjust(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let owner_id = &session.user.id;
    let item = item(&state, owner_id, &id).await?;
    model::adjust(&state.executor, owner_id, &item, None).await?;
    Ok(StatusCode::NO_CONTENT)
}
