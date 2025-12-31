use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::auth::AuthUser;
use crate::routes::AppState;
use crate::template::Template;
use crate::template::ToastErrorTemplate;
use crate::template::filters;

#[derive(askama::Template)]
#[template(path = "profile-account.html")]
pub struct AccountTemplate {
    // pub error_message: Option<String>,
    pub current_path: String,
    pub profile_path: String,
    pub user: AuthUser,
}

// pub async fn page(template: Template, user: AuthUser) -> impl IntoResponse {
//     template.render(AccountTemplate {
//         // error_message: None,
//         current_path: "profile".to_owned(),
//         profile_path: "account".to_owned(),
//         user,
//     })
// }
//
// #[derive(Deserialize)]
// pub struct ActionInput {
//     pub email: String,
// }

// pub async fn action(
//     _template: Template,
//     State(_state): State<AppState>,
//     // Form(input): Form<ActionInput>,
// ) -> impl IntoResponse {
//     ""
// }

#[derive(Deserialize)]
pub struct SetUsernameActionInput {
    pub username: String,
}

pub async fn set_username_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Form(input): Form<SetUsernameActionInput>,
) -> impl IntoResponse {
    if user.username.is_some() {
        return (
            [("ts-swap", "skip")],
            template.render(ToastErrorTemplate {
                original: None,
                message: "Username has already been set.",
                description: None,
            }),
        )
            .into_response();
    }

    let user = crate::try_response!(anyhow_opt:
        imkitchen_user::load(&app.executor, &app.read_db, &user.id),
        template
    );

    crate::try_response!(
        user.set_username(&app.read_db, &app.write_db, input.username),
        template
    );

    "<div ts-trigger=\"load\" ts-action=\"remove\"></div>".into_response()
}
