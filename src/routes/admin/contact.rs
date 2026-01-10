use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, Edge, ReadResult, Value};
use imkitchen_contact::{
    admin::{AdminView, FilterQuery, SortBy},
    global_stat::GlobalStatView,
};
use imkitchen_shared::contact::{Status, Subject};
use serde::Deserialize;
use strum::VariantArray;

use crate::{
    auth::AuthAdmin,
    routes::AppState,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "admin-contact.html")]
pub struct ContactTemplate {
    pub current_path: String,
    pub stat: GlobalStatView,
    pub today: u32,
    pub contacts: ReadResult<AdminView>,
    pub query: PageQuery,
}

impl Default for ContactTemplate {
    fn default() -> Self {
        Self {
            current_path: "contact".to_owned(),
            stat: GlobalStatView::default(),
            contacts: ReadResult::default(),
            today: 0,
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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    Query(query): Query<PageQuery>,
    State(app): State<AppState>,
    user: AuthAdmin,
) -> impl IntoResponse {
    let stat = crate::try_page_response!(
        imkitchen_contact::global_stat::find_global(&app.read_db),
        template
    )
    .unwrap_or_default();

    let now = time::UtcDateTime::now().unix_timestamp() as u64;
    let today_stat = crate::try_page_response!(
        imkitchen_contact::global_stat::find(&app.read_db, now),
        template
    )
    .unwrap_or_default();

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

    let contacts = crate::try_page_response!(
        imkitchen_contact::admin::filter(
            &app.read_db,
            FilterQuery {
                status,
                subject,
                sort_by,
                args: args.limit(20),
            }
        ),
        template
    );

    template
        .render(ContactTemplate {
            stat,
            contacts,
            query: r_query,
            today: today_stat.today,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn mark_read_and_reply(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    user: AuthAdmin,
) -> impl IntoResponse {
    crate::try_response!(app.contact_cmd.mark_read_and_reply(&id, &user.id), template);

    let contact = crate::try_response!(anyhow_opt:
        imkitchen_contact::admin::load(&app.executor, &app.read_db,&id),
        template
    );

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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn resolve(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    user: AuthAdmin,
) -> impl IntoResponse {
    crate::try_response!(app.contact_cmd.resolve(&id, &user.id), template);

    let contact = crate::try_response!(anyhow_opt:
        imkitchen_contact::admin::load(&app.executor, &app.read_db,&id),
        template
    );

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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn reopen(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    user: AuthAdmin,
) -> impl IntoResponse {
    crate::try_response!(app.contact_cmd.reopen(&id, &user.id), template);

    let contact = crate::try_response!(anyhow_opt:
        imkitchen_contact::admin::load(&app.executor, &app.read_db,&id),
        template
    );

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
