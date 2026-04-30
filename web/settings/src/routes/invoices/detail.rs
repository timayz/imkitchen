use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use imkitchen_billing::invoice_user::InvoiceUserView;

use imkitchen_web_shared::{
    AppState,
    auth::AuthUser,
    template::{NotFoundTemplate, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "invoices-detail.html")]
pub struct DetailTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub invoice: InvoiceUserView,
    pub tax_label: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let invoice = imkitchen_web_shared::try_page_response!(opt:
        app.billing.invoice.invoice(id),
        template
    );

    if invoice.user_id != user.id {
        return template.render(NotFoundTemplate);
    }

    let tax_label = if invoice.tax > 0 {
        format!("VAT ({}%)", invoice.tax_rate * 100.0)
    } else {
        "Tax".to_owned()
    };

    template.render(DetailTemplate {
        current_path: "".to_owned(),
        user,
        invoice,
        tax_label,
    })
}
