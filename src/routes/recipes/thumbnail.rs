use crate::{auth::AuthUser, routes::AppState, template::Template};
use axum::{
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use base64::{Engine, engine::general_purpose::STANDARD};

#[tracing::instrument(skip_all)]
pub async fn get(
    State(app): State<AppState>,
    Path((id, device)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match app.recipe_query.find_thumbnail(id, device).await {
        Ok(thumbnail) => match thumbnail {
            Some(thumbnail) => Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/webp")],
                thumbnail.data,
            )),
            _ => Err((StatusCode::NOT_FOUND, "thumbnail not found".to_owned())),
        },
        Err(err) => {
            tracing::error!("{err}");

            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error".to_owned(),
            ));
        }
    }
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-detail-thumbnail.html")]
pub struct ThumbnailTemplate {
    pub data_url: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn upload(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let Some(field) = crate::try_response!(anyhow: multipart.next_field(), template) else {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::User("No file provided".into()))
        , template)
    };
    let content_type = field.content_type().unwrap_or("").to_string();

    // Filter by content type
    let allowed_types = ["image/png", "image/jpeg", "image/webp"];
    if !allowed_types.contains(&content_type.as_str()) {
        return crate::try_response!(sync:
            Err(imkitchen_shared::Error::User(format!("Invalid file type: {content_type}"))),
            template
        );
    }

    let data = crate::try_response!(anyhow: field.bytes(), template);
    crate::try_response!(
        app.recipe_cmd
            .upload_thunmnail(&id, data.to_vec(), &user.id),
        template
    );
    let encoded = STANDARD.encode(&data);
    let data_url = format!("data:{};base64,{}", content_type, encoded);

    template.render(ThumbnailTemplate { data_url })
}
