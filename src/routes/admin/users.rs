use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, Edge, ReadResult, Value};
use imkitchen_user::{
    Role, State as UserState,
    admin::{AdminView, FilterQuery, UserSortBy},
    stat::UserStatRow,
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
    pub stat: UserStatRow,
    pub users: ReadResult<AdminView>,
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

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn page(
    template: Template,
    Query(query): Query<PageQuery>,
    State(app): State<AppState>,
    admin: AuthAdmin,
) -> impl IntoResponse {
    // let stat = crate::try_page_response!(opt: app_state.user_query.find_stat(0), template);
    let stat = UserStatRow::default();

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
        imkitchen_user::admin::filter(
            &app.read_db,
            FilterQuery {
                state,
                sort_by,
                role,
                args: args.limit(20),
            }
        ),
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

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn suspend(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    admin: AuthAdmin,
) -> impl IntoResponse {
    let user = crate::try_response!(anyhow_opt:
        imkitchen_user::load(&app.executor, &app.read_db, &id),
        template
    );
    crate::try_response!(user.suspend(&admin.id), template);

    let user = crate::try_response!(anyhow_opt:
        imkitchen_user::admin::load(&app.executor, &app.read_db, &id),
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
    let user = crate::try_response!(anyhow_opt:
        imkitchen_user::load(&app.executor, &app.read_db, &id),
        template
    );
    crate::try_response!(user.activate(&admin.id), template);

    let user = crate::try_response!(anyhow_opt:
        imkitchen_user::admin::load(&app.executor, &app.read_db, &id),
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
    let subscription = crate::try_response!(anyhow:
        imkitchen_user::subscription::load(&app.executor, &id),
        template
    );
    crate::try_response!(subscription.toggle_life_premium(&admin.id), template);

    let user = crate::try_response!(anyhow_opt:
        imkitchen_user::admin::load(&app.executor, &app.read_db, &id),
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
