use std::str::FromStr;

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
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "admin-users.html")]
pub struct UsersTemplate {
    pub current_path: String,
    pub stat: UserStatRow,
    pub users: ReadResult<UserListRow>,
    pub query: PageQuery,
}

impl Default for UsersTemplate {
    fn default() -> Self {
        Self {
            current_path: "users".to_owned(),
            stat: UserStatRow::default(),
            users: ReadResult::default(),
            query: Default::default(),
        }
    }
}

#[derive(Deserialize, Default, Clone)]
pub struct PageQuery {
    pub first: Option<u16>,
    pub after: Option<Value>,
    pub last: Option<u16>,
    pub before: Option<Value>,
    pub state: Option<String>,
    pub role: Option<String>,
    pub sort_by: Option<String>,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    Query(query): Query<PageQuery>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    let stat = crate::try_page_response!(opt: app_state.user_query.find_stat(0), template);

    let r_query = query.clone();
    let role = Role::from_str(&query.role.unwrap_or("".to_owned())).ok();
    let state = UserState::from_str(&query.state.unwrap_or("".to_owned())).ok();
    let sort_by = UserSortBy::from_str(&query.sort_by.unwrap_or("".to_owned()))
        .unwrap_or(UserSortBy::RecentlyJoined);

    let args = Args {
        first: query.first,
        after: query.after,
        last: query.last,
        before: query.before,
    };

    let users = crate::try_page_response!(
        app_state.user_query.filter(FilterQuery {
            state,
            sort_by,
            role,
            args: args.limit(20),
        }),
        template
    );

    template
        .render(UsersTemplate {
            stat,
            users,
            query: r_query,
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
    crate::try_page_response!(
        app_state.user_command.suspend(&id, &Metadata::by(user.id)),
        template
    );

    let mut user = crate::try_page_response!(opt: app_state.user_query.find(&id), template);

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
    crate::try_page_response!(
        app_state.user_command.activate(&id, &Metadata::by(user.id)),
        template
    );

    let mut user = crate::try_page_response!(opt: app_state.user_query.find(&id), template);

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
    let expire_at = crate::try_page_response!(
        app_state
            .user_subscription_command
            .toggle_life_premium(&id, &Metadata::by(user.id.to_owned())),
        template
    );

    let mut user = crate::try_page_response!(opt: app_state.user_query.find(&id), template);

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
