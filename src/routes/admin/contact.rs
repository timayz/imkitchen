use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, Edge, ReadResult, Value};
use serde::Deserialize;

use crate::{
    auth::AuthAdmin,
    routes::AppState,
    template::{ServerErrorTemplate, Template},
};

#[derive(askama::Template)]
#[template(path = "admin-contact.html")]
pub struct ContactTemplate {
    pub current_path: String,
    // pub stats: AdminUserGlobalStats,
    // pub users: ReadResult<AdminUser>,
}

impl Default for ContactTemplate {
    fn default() -> Self {
        Self {
            current_path: "contact".to_owned(),
            // stats: AdminUserGlobalStats::default(),
            // users: ReadResult::default(),
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
    template: Template<ContactTemplate>,
    // server_error_template: Template<ServerErrorTemplate>,
    Query(query): Query<PageQuery>,
    State(app_state): State<AppState>,

    AuthAdmin(_user): AuthAdmin,
) -> impl IntoResponse {
    // let stats = match query_admin_users_global_stats(&app_state.pool).await {
    //     Ok(stats) => stats,
    //     Err(e) => {
    //         tracing::error!("{e}");
    //
    //         return server_error_template
    //             .render(ServerErrorTemplate)
    //             .into_response();
    //     }
    // };
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

    // let users = match query_admin_users(
    //     &app_state.pool,
    //     AdminUserInput {
    //         status: None,
    //         sort_by: AdminUserSortBy::RecentlyJoined,
    //         account_type: None,
    //         args,
    //     },
    // )
    // .await
    // {
    //     Ok(stats) => stats,
    //     Err(e) => {
    //         tracing::error!("{e}");
    //
    //         return server_error_template
    //             .render(ServerErrorTemplate)
    //             .into_response();
    //     }
    // };

    template
        .render(ContactTemplate {
            // stats,
            // users,
            ..Default::default()
        })
        .into_response()
}
