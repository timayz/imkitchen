//! `/v1/admin/users/*`: what an administrator does with the accounts of
//! others. Every route takes [`Admin`], so anyone else is refused before
//! anything is read; identity then holds its own rules on top: only root
//! makes administrators, nobody deletes root or themself.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use timada_identity::IdentityError;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use validator::Validate;

use crate::auth::{Account, Admin, Role};
use crate::error::{ApiResult, Body};
use crate::openapi::ADMIN;
use crate::server::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(user, delete_user))
        .routes(routes!(set_role))
        .routes(routes!(sign_out))
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
