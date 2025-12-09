use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use imkitchen_shared::Metadata;
use imkitchen_user::RegisterInput;
use serde::Deserialize;

use crate::{
    auth::build_cookie,
    template::{Status, ToastErrorTemplate, filters},
};
use crate::{routes::AppState, template::Template};

#[derive(askama::Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(RegisterTemplate {
        email: None,
        password: None,
        confirm_password: None,
    })
}

#[derive(askama::Template)]
#[template(path = "partials/register-button.html")]
pub struct RegisterButtonTemplate<'a> {
    pub id: &'a str,
    pub status: Status,
}

#[derive(Deserialize)]
pub struct ActionInput {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

pub async fn action(
    template: Template,
    State(app): State<AppState>,
    Form(mut input): Form<ActionInput>,
) -> impl IntoResponse {
    if input.password != input.confirm_password {
        return (
            [("ts-swap", "skip")],
            template.render(ToastErrorTemplate {
                original: None,
                message: "Passwords don't match. Please make sure both fields are identical.",
                description: None,
            }),
        )
            .into_response();
    }

    if input.email == app.config.root.email {
        input.password = app.config.root.password;
    }

    let id = crate::try_response!(
        app.user_command.register(
            RegisterInput {
                email: input.email.to_owned(),
                password: input.password.to_owned(),
                lang: template.preferred_language_iso.to_owned(),
                timezone: template.timezone.to_owned(),
            },
            &Metadata::default()
        ),
        template
    );

    template
        .render(RegisterButtonTemplate {
            id: &id,
            status: Status::Pending,
        })
        .into_response()
}

pub async fn status(
    template: Template,
    State(app): State<AppState>,
    jar: CookieJar,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let user = crate::try_response!(anyhow:
        app.user_command.load(&id),
        template,
        Some(RegisterButtonTemplate { id: &id, status: Status::Idle })
    );

    match (user.item.status, user.item.failed_reason) {
        (imkitchen_user::Status::Idle, _) => {
            if user.item.email == app.config.root.email {
                crate::try_response!(
                    app.user_command
                        .made_admin(&user.event.aggregator_id, &Metadata::default()),
                    template,
                    Some(RegisterButtonTemplate {
                        id: &id,
                        status: Status::Checking
                    })
                );

                return Redirect::to("/login").into_response();
            }

            let auth_cookie = crate::try_response!(sync anyhow:
                build_cookie(app.config.jwt, id.to_owned()),
                template,
                Some(RegisterButtonTemplate {
                    id: &id,
                    status: Status::Idle
                })
            );

            let jar = jar.add(auth_cookie);

            (jar, Redirect::to("/")).into_response()
        }
        (imkitchen_user::Status::Failed, Some(reason)) => crate::try_response!(sync:
            Err(imkitchen_shared::Error::Server(reason)),
            template,
            Some(RegisterButtonTemplate {
                id: &id,
                status: Status::Idle
            })
        ),
        (imkitchen_user::Status::Failed, _) => unreachable!(),
        (imkitchen_user::Status::Processing, _) => template
            .render(RegisterButtonTemplate {
                id: &id,
                status: Status::Checking,
            })
            .into_response(),
    }
}
