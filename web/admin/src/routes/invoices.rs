use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use evento::cursor::{Args, ReadResult, Value};
use imkitchen_billing::invoice_user::{AdminFilterQuery, InvoiceUserView};
use serde::Deserialize;

use imkitchen_web_shared::{
    AppState,
    auth::AuthAdmin,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "admin-invoices.html")]
pub struct InvoicesTemplate {
    pub current_path: String,
    pub invoices: ReadResult<InvoiceUserView>,
    pub query: PageQuery,
}

#[derive(askama::Template)]
#[template(path = "admin-invoices-detail.html")]
pub struct InvoiceDetailTemplate {
    pub current_path: String,
    pub invoice: InvoiceUserView,
    pub tax_label: String,
}

impl Default for InvoicesTemplate {
    fn default() -> Self {
        Self {
            current_path: "invoices".to_owned(),
            invoices: ReadResult::default(),
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
    pub search: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

fn parse_date_to_timestamp(date_str: &str) -> Option<u64> {
    let format = time::macros::format_description!("[year]-[month]-[day]");
    let date = time::Date::parse(date_str, &format).ok()?;
    let time = time::Time::from_hms(0, 0, 0).ok()?;
    let datetime = time::UtcDateTime::new(date, time);
    Some(datetime.unix_timestamp() as u64)
}

fn parse_date_end_to_timestamp(date_str: &str) -> Option<u64> {
    let format = time::macros::format_description!("[year]-[month]-[day]");
    let date = time::Date::parse(date_str, &format).ok()?;
    let time = time::Time::from_hms(23, 59, 59).ok()?;
    let datetime = time::UtcDateTime::new(date, time);
    Some(datetime.unix_timestamp() as u64)
}

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn page(
    template: Template,
    Query(query): Query<PageQuery>,
    State(app): State<AppState>,
    admin: AuthAdmin,
) -> impl IntoResponse {
    let r_query = query.clone();

    let search = match query.search.as_deref() {
        Some("") | None => None,
        Some(s) => Some(s.to_owned()),
    };

    let date_from = query
        .date_from
        .as_deref()
        .filter(|s| !s.is_empty())
        .and_then(parse_date_to_timestamp);

    let date_to = query
        .date_to
        .as_deref()
        .filter(|s| !s.is_empty())
        .and_then(parse_date_end_to_timestamp);

    let args = Args {
        first: query.first,
        after: query.after,
        last: query.last,
        before: query.before,
    };

    let invoices = imkitchen_web_shared::try_page_response!(
        app.billing.invoice.filter_invoice_admin(AdminFilterQuery {
            search,
            date_from,
            date_to,
            args: args.limit(20),
        }),
        template
    );

    template
        .render(InvoicesTemplate {
            invoices,
            query: r_query,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn detail(
    template: Template,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
    admin: AuthAdmin,
) -> impl IntoResponse {
    let invoice = imkitchen_web_shared::try_page_response!(opt:
        app.billing.invoice.invoice(id),
        template
    );

    let tax_label = if invoice.is_vat {
        format!("VAT ({}%)", invoice.tax_rate * 100.0)
    } else {
        "Tax".to_owned()
    };

    template
        .render(InvoiceDetailTemplate {
            current_path: "invoices".to_owned(),
            invoice,
            tax_label,
        })
        .into_response()
}
