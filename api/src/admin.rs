//! `/admin/users/*`: what an administrator does with the accounts of
//! others. Every route takes [`Admin`], so anyone else is refused before
//! anything is read; identity then holds its own rules on top: only root
//! makes administrators, nobody deletes root or themself.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use timada_identity::IdentityError;

use crate::auth::{Account, Admin, Role};
use crate::error::{ApiResult, Body};
use crate::server::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/users/{id}", get(user).delete(delete_user))
        .route("/admin/users/{id}/role", put(set_role))
        .route("/admin/users/{id}/sign-out", post(sign_out))
}

/// `GET /admin/users/{id}`
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

#[derive(Deserialize)]
struct NewRole {
    role: Role,
}

/// `PUT /admin/users/{id}/role`: makes a user an administrator, or one no
/// more. Root's to do.
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

/// `DELETE /admin/users/{id}`: the account is gone and its sessions end;
/// the person may register again.
async fn delete_user(
    State(state): State<AppState>,
    Admin(admin): Admin,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    state.identity().delete_user(&admin.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /admin/users/{id}/sign-out`: ends every session of the user.
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
