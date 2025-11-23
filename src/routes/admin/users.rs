use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, Edge, ReadResult, Value};
use imkitchen_shared::Metadata;
use imkitchen_user::{FilterQuery, Role, State as UserState, UserListRow, UserSortBy, UserStatRow};
use serde::Deserialize;
use strum::VariantArray;

use crate::{
    auth::AuthAdmin,
    routes::AppState,
    template::{NotFoundTemplate, ServerErrorTemplate, Template},
};

#[derive(askama::Template)]
#[template(path = "admin-users.html")]
pub struct UsersTemplate {
    pub current_path: String,
    pub stat: UserStatRow,
    pub users: ReadResult<UserListRow>,
}

impl Default for UsersTemplate {
    fn default() -> Self {
        Self {
            current_path: "users".to_owned(),
            stat: UserStatRow::default(),
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
    not_found: Template<NotFoundTemplate>,
    Query(query): Query<PageQuery>,
    State(app_state): State<AppState>,

    AuthAdmin(_user): AuthAdmin,
) -> impl IntoResponse {
    let stat = match app_state.user_query.find_stat(0).await {
        Ok(Some(stats)) => stats,
        Ok(_) => return not_found.render(NotFoundTemplate).into_response(),
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

    let users = match app_state
        .user_query
        .filter(FilterQuery {
            state: None,
            sort_by: UserSortBy::RecentlyJoined,
            role: None,
            args: args.limit(20),
        })
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
            stat,
            users,
            ..Default::default()
        })
        .into_response()
}

pub async fn suspend(
    template: Template<UsersTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    not_found: Template<NotFoundTemplate>,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app_state
        .user_command
        .suspend(&id, &Metadata::by(user.id))
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut user = match app_state.user_query.find(&id).await {
        Ok(Some(u)) => u,
        Ok(_) => return not_found.render(NotFoundTemplate).into_response(),
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    user.state.0 = UserState::Suspended;

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
    not_found: Template<NotFoundTemplate>,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app_state
        .user_command
        .activate(&id, &Metadata::by(user.id))
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut user = match app_state.user_query.find(&id).await {
        Ok(Some(u)) => u,
        Ok(_) => return not_found.render(NotFoundTemplate).into_response(),
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    user.state.0 = UserState::Active;

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
    not_found: Template<NotFoundTemplate>,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    let expire_at = match app_state
        .user_subscription_command
        .toggle_life_premium(&id, &Metadata::by(user.id.to_owned()))
        .await
    {
        Ok(expire_at) => expire_at,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    let mut user = match app_state.user_query.find(&id).await {
        Ok(Some(u)) => u,
        Ok(_) => return not_found.render(NotFoundTemplate).into_response(),
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    user.subscription_expire_at = expire_at;

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
