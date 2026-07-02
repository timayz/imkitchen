use axum::extract::State;
use axum::response::IntoResponse;

use imkitchen_identity::password::RequestInput;
use imkitchen_web_shared::AppState;
use imkitchen_web_shared::auth::AuthUser;
use imkitchen_web_shared::template::Template;
use imkitchen_web_shared::template::ToastSuccessTemplate;
use imkitchen_web_shared::template::filters;

#[derive(askama::Template)]
#[template(path = "settings-account.html")]
pub struct SecurityTemplate {
    pub current_path: String,
    pub settings_path: String,
    pub user: AuthUser,
}

pub async fn page(template: Template, user: AuthUser) -> impl IntoResponse {
    template.render(SecurityTemplate {
        current_path: "settings".to_owned(),
        settings_path: "account".to_owned(),
        user,
    })
}

pub async fn action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(
        app.identity.password.request(RequestInput {
            email: user.email.to_owned(),
            lang: template.preferred_language_iso.to_owned(),
            host: app.config.server.url.to_owned(),
        }),
        template
    );

    template
        .render(ToastSuccessTemplate {
            original: None,
            message: "We've emailed you a secure link to reset your password.",
            description: None,
        })
        .into_response()
}
