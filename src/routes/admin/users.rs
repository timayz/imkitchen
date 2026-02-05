use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, Edge, ReadResult, Value};
use imkitchen_shared::user::{Role, State as UserState};
use imkitchen_user::{
    admin::{AdminView, FilterQuery, UserSortBy},
    global_stat::{FilterQuery as FilterQueryStat, GlobalStatView},
};
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
    pub stat: GlobalStatView,
    pub users: ReadResult<AdminView>,
    pub query: PageQuery,
    pub total_percent: i64,
    pub suspended_percent: i64,
    pub premium_percent: i64,
}

impl Default for UsersTemplate {
    fn default() -> Self {
        Self {
            current_path: "users".to_owned(),
            stat: GlobalStatView::default(),
            users: ReadResult::default(),
            query: Default::default(),
            premium_percent: 0,
            suspended_percent: 0,
            total_percent: 0,
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
    pub search: Option<String>,
    pub role: Option<String>,
    pub sort_by: Option<String>,
}

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn page(
    template: Template,
    Query(query): Query<PageQuery>,
    State(app): State<AppState>,
    admin: AuthAdmin,
) -> impl IntoResponse {
    let stat =
        crate::try_page_response!(app.user_query.find_global(), template).unwrap_or_default();

    let stats = crate::try_page_response!(
        app.user_query.filter_global(FilterQueryStat {
            args: Args::backward(1, None)
        }),
        template
    );

    let current_stat = stats
        .edges
        .first()
        .map(|e| e.node.clone())
        .unwrap_or_default();

    let total_percent = current_stat.total_percent(&stat);
    let suspended_percent = stat.suspended_percent();
    let premium_percent = current_stat.premium_percent(&stat);

    let r_query = query.clone();
    let role = Role::from_str(&query.role.unwrap_or("".to_owned())).ok();
    let state = UserState::from_str(&query.state.unwrap_or("".to_owned())).ok();
    let sort_by = UserSortBy::from_str(&query.sort_by.unwrap_or("".to_owned()))
        .unwrap_or(UserSortBy::RecentlyJoined);

    let search = if let Some("") = query.search.as_deref() {
        None
    } else {
        query.search
    };

    let args = Args {
        first: query.first,
        after: query.after,
        last: query.last,
        before: query.before,
    };

    let users = crate::try_page_response!(
        app.user_query.filter_admin(FilterQuery {
            state,
            sort_by,
            role,
            args: args.limit(20),
            search,
        }),
        template
    );

    template
        .render(UsersTemplate {
            stat,
            users,
            query: r_query,
            suspended_percent,
            total_percent,
            premium_percent,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn suspend(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    admin: AuthAdmin,
) -> impl IntoResponse {
    crate::try_response!(app.user_cmd.suspend(&id, &admin.id), template);

    let user = crate::try_response!(anyhow_opt:
        app.user_query.admin( &id),
        template
    );

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

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn activate(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    admin: AuthAdmin,
) -> impl IntoResponse {
    crate::try_response!(app.user_cmd.activate(&id, &admin.id), template);

    let user = crate::try_response!(anyhow_opt:
        app.user_query.admin(&id),
        template
    );

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

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn toggle_premium(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    admin: AuthAdmin,
) -> impl IntoResponse {
    crate::try_response!(
        app.user_cmd
            .subscription
            .toggle_life_premium(&id, &admin.id),
        template
    );

    let user = crate::try_response!(anyhow_opt:
        app.user_query.admin(&id),
        template
    );

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
