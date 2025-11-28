use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, Edge, ReadResult, Value};
use imkitchen_contact::{ContactRow, FilterQuery, SortBy, Stat, Status, Subject};
use imkitchen_shared::Metadata;
use serde::Deserialize;
use strum::VariantArray;

use crate::{
    auth::AuthAdmin,
    routes::AppState,
    template::{NotFoundTemplate, ServerErrorTemplate, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "admin-contact.html")]
pub struct ContactTemplate {
    pub current_path: String,
    pub stats: Stat,
    pub contacts: ReadResult<ContactRow>,
    pub query: PageQuery,
}

impl Default for ContactTemplate {
    fn default() -> Self {
        Self {
            current_path: "contact".to_owned(),
            stats: Stat::default(),
            contacts: ReadResult::default(),
            query: Default::default(),
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct PageQuery {
    pub first: Option<u16>,
    pub after: Option<Value>,
    pub last: Option<u16>,
    pub before: Option<Value>,
    pub status: Option<String>,
    pub subject: Option<String>,
    pub sort_by: Option<String>,
}

pub async fn page(
    template: Template<ContactTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    Query(query): Query<PageQuery>,
    State(app): State<AppState>,
    AuthAdmin(_user): AuthAdmin,
) -> impl IntoResponse {
    let stats = match app.contact_query.find_stat(0).await {
        Ok(stats) => stats.unwrap_or_default(),
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    let r_query = query.clone();
    let subject = Subject::from_str(&query.subject.unwrap_or("".to_owned())).ok();
    let status = Status::from_str(&query.status.unwrap_or("".to_owned())).ok();
    let sort_by =
        SortBy::from_str(&query.sort_by.unwrap_or("".to_owned())).unwrap_or(SortBy::MostRecent);

    let args = Args {
        first: query.first,
        after: query.after,
        last: query.last,
        before: query.before,
    };

    let contacts = match app
        .contact_query
        .filter(FilterQuery {
            status,
            subject,
            sort_by,
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
        .render(ContactTemplate {
            stats,
            contacts,
            query: r_query,
            ..Default::default()
        })
        .into_response()
}

pub async fn mark_read_and_reply(
    template: Template<ContactTemplate>,
    not_found: Template<NotFoundTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app
        .contact_command
        .mark_read_and_reply(&id, &Metadata::by(user.id))
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut contact = match app.contact_query.find(&id).await {
        Ok(Some(u)) => u,
        Ok(_) => return not_found.render(NotFoundTemplate).into_response(),
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    contact.status.0 = Status::Read;

    let contacts = ReadResult {
        page_info: Default::default(),
        edges: vec![Edge {
            cursor: "".to_owned().into(),
            node: contact,
        }],
    };

    template
        .render(ContactTemplate {
            contacts,
            ..Default::default()
        })
        .into_response()
}

pub async fn resolve(
    template: Template<ContactTemplate>,
    not_found: Template<NotFoundTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app
        .contact_command
        .resolve(&id, &Metadata::by(user.id))
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut contact = match app.contact_query.find(&id).await {
        Ok(Some(u)) => u,
        Ok(_) => return not_found.render(NotFoundTemplate).into_response(),
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    contact.status.0 = Status::Resolved;

    let contacts = ReadResult {
        page_info: Default::default(),
        edges: vec![Edge {
            cursor: "".to_owned().into(),
            node: contact,
        }],
    };

    template
        .render(ContactTemplate {
            contacts,
            ..Default::default()
        })
        .into_response()
}

pub async fn reopen(
    template: Template<ContactTemplate>,
    not_found: Template<NotFoundTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app
        .contact_command
        .reopen(&id, &Metadata::by(user.id))
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut contact = match app.contact_query.find(&id).await {
        Ok(Some(u)) => u,
        Ok(_) => return not_found.render(NotFoundTemplate).into_response(),
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    contact.status.0 = Status::Read;

    let contacts = ReadResult {
        page_info: Default::default(),
        edges: vec![Edge {
            cursor: "".to_owned().into(),
            node: contact,
        }],
    };

    template
        .render(ContactTemplate {
            contacts,
            ..Default::default()
        })
        .into_response()
}
