use std::str::FromStr;

use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use imkitchen_contact::{Subject, SubmitContactFormInput};
use imkitchen_shared::Metadata;
use serde::Deserialize;
use strum::VariantArray;

use crate::{
    routes::AppState,
    template::{Template, ToastErrorTemplate, ToastSuccessTemplate, filters},
};

#[derive(askama::Template)]
#[template(path = "contact.html")]
pub struct ContactTemplate;

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(ContactTemplate)
}

#[derive(Deserialize)]
pub struct ActionInput {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
}

pub async fn action(
    template: Template,
    State(app_state): State<AppState>,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    let Ok(subject) = Subject::from_str(&input.subject) else {
        return template.render(ToastErrorTemplate {
            original: None,
            message: "invalid subject",
            description: None,
        });
    };

    crate::try_response!(
        app_state.contact_command.submit_contact_form(
            SubmitContactFormInput {
                name: input.name,
                email: input.email,
                subject,
                message: input.message,
            },
            &Metadata::default(),
        ),
        template
    );

    template
        .render(ToastSuccessTemplate {
            original: None,
            message: "Contact form submitted successfully",
            description: None,
        })
        .into_response()
}
