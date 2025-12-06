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
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "contact.html")]
pub struct ContactTemplate {
    pub error_message: Option<String>,
    pub succeeded: bool,
}

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(ContactTemplate {
        succeeded: false,
        error_message: None,
    })
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
        return template.render(ContactTemplate {
            error_message: Some("invalide subject".into()),
            succeeded: false,
        });
    };

    match app_state
        .contact_command
        .submit_contact_form(
            SubmitContactFormInput {
                name: input.name,
                email: input.email,
                subject,
                message: input.message,
            },
            &Metadata::default(),
        )
        .await
    {
        Ok(_) => template.render(ContactTemplate {
            succeeded: true,
            error_message: None,
        }),
        Err(e) => template.render(ContactTemplate {
            succeeded: false,
            error_message: Some(e.to_string()),
        }),
    }
}
