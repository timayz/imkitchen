use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, Edge, ReadResult, Value};
use imkitchen_shared::Metadata;
use imkitchen_user::{FilterQuery, Role, State as UserState, UserListRow, UserSortBy, UserStatRow};
use serde::Deserialize;
use strum::VariantArray;

use crate::{auth::AuthAdmin, routes::AppState, template::Template};

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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    Query(query): Query<PageQuery>,
    State(app_state): State<AppState>,

    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    let stat = crate::try_anyhow_opt_response!(app_state.user_query.find_stat(0), template);

    let args = Args {
        first: query.first,
        after: query.after,
        last: query.last,
        before: query.before,
    };

    let users = crate::try_anyhow_response!(
        app_state.user_query.filter(FilterQuery {
            state: None,
            sort_by: UserSortBy::RecentlyJoined,
            role: None,
            args: args.limit(20),
        }),
        template
    );

    template
        .render(UsersTemplate {
            stat,
            users,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn suspend(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    crate::try_anyhow_response!(
        app_state.user_command.suspend(&id, &Metadata::by(user.id)),
        template
    );

    let mut user = crate::try_anyhow_opt_response!(app_state.user_query.find(&id), template);

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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn activate(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    crate::try_anyhow_response!(
        app_state.user_command.activate(&id, &Metadata::by(user.id)),
        template
    );

    let mut user = crate::try_anyhow_opt_response!(app_state.user_query.find(&id), template);

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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn toggle_premium(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    let expire_at = crate::try_anyhow_response!(
        app_state
            .user_subscription_command
            .toggle_life_premium(&id, &Metadata::by(user.id.to_owned())),
        template
    );

    let mut user = crate::try_anyhow_opt_response!(app_state.user_query.find(&id), template);

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
