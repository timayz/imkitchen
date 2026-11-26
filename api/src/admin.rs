//! `/v1/admin/*`: what an administrator does with the accounts of others
//! and with what they keep. Every route takes [`Admin`], so anyone else is
//! refused before anything is read; identity then holds its own rules on
//! top: only root makes administrators, nobody deletes root or themself.
//! Suspending follows them ([`member`]).
//!
//! A recipe is its owner's alone, and no route but these reads or writes
//! someone else's: an administrator deletes one, or takes a picture off
//! every recipe that shows it.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use timada_identity::{IdentityError, UserState};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use validator::Validate;

use crate::auth::{Account, Admin, Role, Session};
use crate::error::{ApiError, ApiResult, Body};
use crate::images::{self, IMAGE_NOT_FOUND};
use crate::openapi::ADMIN;
use crate::recipes;
use crate::server::AppState;
use crate::suspension;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(user, delete_user))
        .routes(routes!(set_role))
        .routes(routes!(sign_out))
        .routes(routes!(suspend, reactivate))
        .routes(routes!(delete_recipe))
        .routes(routes!(delete_image))
}

/// The account of a user
#[utoipa::path(
    get,
    path = "/admin/users/{id}",
    tag = ADMIN,
    security(("session" = [])),
    params(("id" = String, Path, description = "The user's id")),
    responses((status = OK, body = Account))
)]
async fn user(
    State(state): State<AppState>,
    Admin(_): Admin,
    Path(id): Path<String>,
) -> ApiResult<Json<Account>> {
    let user = state
        .identity()
        .load_user(id)
        .await?
        .ok_or(IdentityError::UserNotFound)?;
    Ok(Json(Account::from(&user)))
}

#[derive(Deserialize, Validate, ToSchema)]
struct NewRole {
    role: Role,
}

/// Makes a user an administrator, or one no more
///
/// Root's to do.
#[utoipa::path(
    put,
    path = "/admin/users/{id}/role",
    tag = ADMIN,
    security(("session" = [])),
    params(("id" = String, Path, description = "The user's id")),
    request_body = NewRole,
    responses((status = NO_CONTENT, description = "The role is set"))
)]
async fn set_role(
    State(state): State<AppState>,
    Admin(admin): Admin,
    Path(id): Path<String>,
    Body(form): Body<NewRole>,
) -> ApiResult<StatusCode> {
    state
        .identity()
        .set_admin(&admin.user.id, &id, form.role == Role::Admin)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes the account of a user
///
/// The account is gone and its sessions end; the person may register again.
#[utoipa::path(
    delete,
    path = "/admin/users/{id}",
    tag = ADMIN,
    security(("session" = [])),
    params(("id" = String, Path, description = "The user's id")),
    responses((status = NO_CONTENT, description = "The account is deleted"))
)]
async fn delete_user(
    State(state): State<AppState>,
    Admin(admin): Admin,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    state.identity().delete_user(&admin.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Ends every session of a user
#[utoipa::path(
    post,
    path = "/admin/users/{id}/sign-out",
    tag = ADMIN,
    security(("session" = [])),
    params(("id" = String, Path, description = "The user's id")),
    responses((status = NO_CONTENT, description = "The user is signed out everywhere"))
)]
async fn sign_out(
    State(state): State<AppState>,
    Admin(admin): Admin,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    state
        .identity()
        .revoke_sessions(&admin.user.id, &id, None)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The user `id`, when `admin` may keep them out or let them in again: as
/// with deleting, an administrator does it to members and root to
/// administrators too, nobody to themself or to root.
async fn member(state: &AppState, admin: &Session, id: &str) -> ApiResult<UserState> {
    let user = state
        .identity()
        .load_user(id)
        .await?
        .ok_or(IdentityError::UserNotFound)?;
    let allowed =
        user.id != admin.user.id && !user.is_root() && (!user.is_admin || admin.user.is_root());
    if !allowed {
        return Err(IdentityError::Forbidden.into());
    }
    Ok(user)
}

/// Suspends a user
///
/// Their account and all that is theirs stay, but every session of theirs
/// ends and signing in is refused (`account_suspended`) until they are
/// reactivated. A user who is suspended already stays so.
#[utoipa::path(
    put,
    path = "/admin/users/{id}/suspension",
    tag = ADMIN,
    security(("session" = [])),
    params(("id" = String, Path, description = "The user's id")),
    responses((status = NO_CONTENT, description = "The user is suspended"))
)]
async fn suspend(
    State(state): State<AppState>,
    Admin(admin): Admin,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let user = member(&state, &admin, &id).await?;
    suspension::suspend(&state.executor, &user.id, &admin.user.id).await?;
    // Whatever the suspension wrote: one whose sessions were not ended the
    // first time ends them when asked again.
    state
        .identity()
        .revoke_sessions(&admin.user.id, &user.id, None)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Reactivates a user who is suspended
///
/// They sign in again as before. A user who is not suspended stays so.
#[utoipa::path(
    delete,
    path = "/admin/users/{id}/suspension",
    tag = ADMIN,
    security(("session" = [])),
    params(("id" = String, Path, description = "The user's id")),
    responses((status = NO_CONTENT, description = "The user is suspended no more"))
)]
async fn reactivate(
    State(state): State<AppState>,
    Admin(admin): Admin,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let user = member(&state, &admin, &id).await?;
    suspension::reactivate(&state.executor, &user.id, &admin.user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes a recipe, whoever's it is
///
/// The recipe is gone for good, as when its owner deletes it: it leaves
/// the plats and the list it was on a moment later.
#[utoipa::path(
    delete,
    path = "/admin/recipes/{id}",
    tag = ADMIN,
    security(("session" = [])),
    params(("id" = String, Path, description = "The recipe's id")),
    responses((status = NO_CONTENT, description = "The recipe is deleted"))
)]
async fn delete_recipe(
    State(state): State<AppState>,
    Admin(_): Admin,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    recipes::remove(&state.executor, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Takes a picture off every recipe that shows it
///
/// The recipes stay, shown without a picture, and the files of the picture
/// go once the last of them let go of it. A recipe given the picture a
/// moment ago may still show it: asked again, it is taken off that one
/// too. Nothing keeps the same picture from being given to a recipe again.
#[utoipa::path(
    delete,
    path = "/admin/images/{id}",
    tag = ADMIN,
    security(("session" = [])),
    params(("id" = String, Path, description = "The picture's id: the SHA-256 it is known under")),
    responses((status = NO_CONTENT, description = "The recipes show the picture no more"))
)]
async fn delete_image(
    State(state): State<AppState>,
    Admin(_): Admin,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    if !images::is_stored(&state.executor, &id).await? {
        return Err(ApiError::refused(StatusCode::NOT_FOUND, IMAGE_NOT_FOUND));
    }
    recipes::remove_image_shown(&state.executor, &state.db, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
