use axum::{
    extract::{Multipart, Path, State},
    response::IntoResponse,
};

use imkitchen_web_shared::{
    AdminImportProgress, AppState,
    auth::AuthAdmin,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "admin-recipes-import.html")]
pub struct ImportTemplate {
    pub current_path: String,
}

impl Default for ImportTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
        }
    }
}

#[derive(askama::Template)]
#[template(path = "partials/admin-recipes-importing.html")]
pub struct ImportingTemplate {
    pub job_id: String,
}

#[derive(askama::Template)]
#[template(path = "partials/admin-recipes-importing-status.html")]
pub struct ImportingStatusTemplate {
    pub job_id: String,
    pub result: Option<AdminImportProgress>,
}

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn page(template: Template, admin: AuthAdmin) -> impl IntoResponse {
    tracing::debug!("admin {} opened recipe import", admin.id);
    template.render(ImportTemplate::default())
}

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn action(
    template: Template,
    State(app): State<AppState>,
    admin: AuthAdmin,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let Some(field) = imkitchen_web_shared::try_response!(anyhow: multipart.next_field(), template)
    else {
        imkitchen_web_shared::try_response!(sync:
            Err(imkitchen_core::Error::User("No file provided".into())),
            template
        )
    };

    let data = imkitchen_web_shared::try_response!(anyhow: field.bytes(), template);
    let bytes = data.to_vec();

    let job_id = ulid::Ulid::new().to_string();
    {
        let mut jobs = app.import_jobs.lock().unwrap_or_else(|e| e.into_inner());
        jobs.insert(job_id.clone(), AdminImportProgress::default());
    }

    let app = app.clone();
    let admin_id = admin.id.clone();
    let job = job_id.clone();
    tokio::spawn(async move {
        let password = app.config.root.password.clone();
        let progress = crate::import::process_zip(
            &app.identity,
            &app.core.recipe,
            &admin_id,
            &password,
            bytes,
        )
        .await;
        let mut jobs = app.import_jobs.lock().unwrap_or_else(|e| e.into_inner());
        jobs.insert(job, progress);
    });

    template.render(ImportingTemplate { job_id })
}

#[tracing::instrument(skip_all, fields(admin = admin.id))]
pub async fn status(
    template: Template,
    State(app): State<AppState>,
    admin: AuthAdmin,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    tracing::debug!("admin {} polling import {id}", admin.id);

    let result = {
        let mut jobs = app.import_jobs.lock().unwrap_or_else(|e| e.into_inner());
        match jobs.get(&id) {
            // Finished: hand back the result once and drop it from the registry.
            Some(p) if p.done => jobs.remove(&id),
            // Still running: keep polling.
            Some(_) => None,
            // Unknown job (server restarted, or already consumed): stop polling.
            None => Some(AdminImportProgress {
                done: true,
                ..Default::default()
            }),
        }
    };

    template.render(ImportingStatusTemplate { job_id: id, result })
}
