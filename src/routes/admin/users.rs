use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, Edge, ReadResult, Value};
use imkitchen_shared::Metadata;
use serde::Deserialize;

use crate::{
    auth::AuthAdmin,
    query::{
        AdminUser, AdminUserAccountType, AdminUserGlobalStats, AdminUserInput, AdminUserSortBy,
        AdminUserStatus, query_admin_user_by_id, query_admin_users, query_admin_users_global_stats,
    },
    routes::AppState,
    template::{ServerErrorTemplate, Template},
};

#[derive(askama::Template)]
#[template(path = "admin-users.html")]
pub struct UsersTemplate {
    pub current_path: String,
    pub stats: AdminUserGlobalStats,
    pub users: ReadResult<AdminUser>,
}

impl Default for UsersTemplate {
    fn default() -> Self {
        Self {
            current_path: "users".to_owned(),
            stats: AdminUserGlobalStats::default(),
            users: ReadResult::default(),
        }
    }
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub first: Option<u16>,
    pub after: Option<Value>,
    pub last: Option<u16>,
    pub before: Option<Value>,
}

pub async fn page(
    template: Template<UsersTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    Query(query): Query<PageQuery>,
    State(app_state): State<AppState>,

    AuthAdmin(_user): AuthAdmin,
) -> impl IntoResponse {
    let stats = match query_admin_users_global_stats(&app_state.pool).await {
        Ok(stats) => stats,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };
    let args = Args {
        first: query.first,
        after: query.after,
        last: query.last,
        before: query.before,
    };

    let args = if args.is_backward() {
        Args::backward(args.last.unwrap_or(20).min(40), args.before)
    } else {
        Args::forward(args.first.unwrap_or(20).min(40), args.after)
    };

    let users = match query_admin_users(
        &app_state.pool,
        AdminUserInput {
            status: None,
            sort_by: AdminUserSortBy::RecentlyJoined,
            account_type: None,
            args,
        },
    )
    .await
    {
        Ok(stats) => stats,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    template
        .render(UsersTemplate {
            stats,
            users,
            ..Default::default()
        })
        .into_response()
}

pub async fn suspend(
    template: Template<UsersTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(_user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app_state
        .user_command
        .suspend(
            imkitchen_user::SuspendInput { id: id.to_owned() },
            Metadata::by(id.to_owned()),
        )
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut user = match query_admin_user_by_id(&app_state.pool, id).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    user.status = AdminUserStatus::Suspended.to_string();

    let users = ReadResult {
        page_info: Default::default(),
        edges: vec![Edge {
            cursor: "".to_owned().into(),
            node: user,
        }],
    };

    template
        .render(UsersTemplate {
            users,
            ..Default::default()
        })
        .into_response()
}

pub async fn activate(
    template: Template<UsersTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(_user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app_state
        .user_command
        .activate(
            imkitchen_user::ActivateInput { id: id.to_owned() },
            Metadata::by(id.to_owned()),
        )
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut user = match query_admin_user_by_id(&app_state.pool, id).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    user.status = AdminUserStatus::Active.to_string();

    let users = ReadResult {
        page_info: Default::default(),
        edges: vec![Edge {
            cursor: "".to_owned().into(),
            node: user,
        }],
    };

    template
        .render(UsersTemplate {
            users,
            ..Default::default()
        })
        .into_response()
}

pub async fn toggle_premium(
    template: Template<UsersTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(_user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app_state
        .user_command
        .toggle_life_premium(
            imkitchen_user::ToggleLifePremiumInput { id: id.to_owned() },
            Metadata::by(id.to_owned()),
        )
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut user = match query_admin_user_by_id(&app_state.pool, id).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    user.account_type = if user.is_free_tier() {
        AdminUserAccountType::Premium.to_string()
    } else if user.is_premium() {
        AdminUserAccountType::FreeTier.to_string()
    } else {
        user.account_type
    };

    let users = ReadResult {
        page_info: Default::default(),
        edges: vec![Edge {
            cursor: "".to_owned().into(),
            node: user,
        }],
    };

    template
        .render(UsersTemplate {
            users,
            ..Default::default()
        })
        .into_response()
}
