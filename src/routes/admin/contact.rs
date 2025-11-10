use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, Edge, ReadResult, Value};
use imkitchen_contact::{ContactStatus, ContactSubject};
use imkitchen_shared::Metadata;
use serde::Deserialize;

use crate::{
    auth::AuthAdmin,
    query::{
        Contact, ContactGlobalStats, ContactInput, ContactSortBy, query_contact_by_id,
        query_contact_global_stats, query_contacts,
    },
    routes::AppState,
    template::{ServerErrorTemplate, Template},
};

#[derive(askama::Template)]
#[template(path = "admin-contact.html")]
pub struct ContactTemplate {
    pub current_path: String,
    pub stats: ContactGlobalStats,
    pub contacts: ReadResult<Contact>,
}

impl Default for ContactTemplate {
    fn default() -> Self {
        Self {
            current_path: "contact".to_owned(),
            stats: ContactGlobalStats::default(),
            contacts: ReadResult::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
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
    State(app_state): State<AppState>,
    AuthAdmin(_user): AuthAdmin,
) -> impl IntoResponse {
    let stats = match query_contact_global_stats(&app_state.pool).await {
        Ok(stats) => stats,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    let subject = ContactSubject::from_str(&query.subject.unwrap_or("".to_owned())).ok();
    let status = ContactStatus::from_str(&query.status.unwrap_or("".to_owned())).ok();
    let sort_by = ContactSortBy::from_str(&query.sort_by.unwrap_or("".to_owned()))
        .unwrap_or(ContactSortBy::MostRecent);

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

    let contacts = match query_contacts(
        &app_state.pool,
        ContactInput {
            status,
            subject,
            sort_by,
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
        .render(ContactTemplate {
            stats,
            contacts,
            ..Default::default()
        })
        .into_response()
}

pub async fn mark_read_and_reply(
    template: Template<ContactTemplate>,
    server_error_template: Template<ServerErrorTemplate>,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app_state
        .contact_command
        .mark_read_and_reply(&id, Metadata::by(user.id))
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut contact = match query_contact_by_id(&app_state.pool, id).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    contact.status = ContactStatus::Read.to_string();

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
    server_error_template: Template<ServerErrorTemplate>,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app_state
        .contact_command
        .resolve(&id, Metadata::by(user.id))
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut contact = match query_contact_by_id(&app_state.pool, id).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    contact.status = ContactStatus::Resolved.to_string();

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
    server_error_template: Template<ServerErrorTemplate>,
    Path((id,)): Path<(String,)>,
    State(app_state): State<AppState>,
    AuthAdmin(user): AuthAdmin,
) -> impl IntoResponse {
    if let Err(e) = app_state
        .contact_command
        .reopen(&id, Metadata::by(user.id.to_owned()))
        .await
    {
        tracing::error!("{e}");

        return server_error_template
            .render(ServerErrorTemplate)
            .into_response();
    }

    let mut contact = match query_contact_by_id(&app_state.pool, id).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("{e}");

            return server_error_template
                .render(ServerErrorTemplate)
                .into_response();
        }
    };

    contact.status = ContactStatus::Read.to_string();

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
